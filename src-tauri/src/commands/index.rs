use crate::models::AddonIndex;
use crate::services::database;
use crate::state::AppState;
use chrono::Utc;
use tauri::State;

/// Default index URL (can be overridden in settings)
const DEFAULT_INDEX_URL: &str = "https://xop.co/eso-addon-index/";

/// Fetch the addon index (from cache or remote)
#[tauri::command]
pub async fn fetch_index(
    force: Option<bool>,
    state: State<'_, AppState>,
) -> Result<AddonIndex, String> {
    let force = force.unwrap_or(false);

    // Check cache first (unless force refresh) - scope the lock
    let (cached_index, index_url) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;

        let cached = if !force {
            if let Ok(Some((data, fetched_at, _))) = database::get_cached_index(&conn) {
                // Check if cache is less than 1 hour old
                if let Ok(fetched) = chrono::DateTime::parse_from_rfc3339(&fetched_at) {
                    let age = Utc::now().signed_duration_since(fetched);
                    if age.num_hours() < 1 {
                        serde_json::from_str::<AddonIndex>(&data).ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let url = database::get_setting(&conn, "index_url")
            .ok()
            .flatten()
            .unwrap_or_else(|| DEFAULT_INDEX_URL.to_string());

        (cached, url)
    }; // Lock is dropped here

    // Return cached index if valid
    if let Some(index) = cached_index {
        return Ok(index);
    }

    // Fetch from remote
    let client = reqwest::Client::new();
    let response = client
        .get(&index_url)
        .header("User-Agent", "eso-addon-manager")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch index: {}", e))?;

    let etag = response
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let data = response
        .text()
        .await
        .map_err(|e| format!("Failed to read index: {}", e))?;

    // Parse the index
    let mut index: AddonIndex =
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse index: {}", e))?;

    // Update fetched_at
    index.fetched_at = Some(Utc::now().to_rfc3339());

    // Cache the index - acquire lock again
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        database::update_cached_index(&conn, &data, etag.as_deref())
            .map_err(|e| format!("Failed to cache index: {}", e))?;
    }

    Ok(index)
}

/// Get the cached index without fetching
#[tauri::command]
pub async fn get_cached_index(state: State<'_, AppState>) -> Result<Option<AddonIndex>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let cached = database::get_cached_index(&conn).map_err(|e| e.to_string())?;

    match cached {
        Some((data, _, _)) => {
            let index: AddonIndex =
                serde_json::from_str(&data).map_err(|e| format!("Failed to parse index: {}", e))?;
            Ok(Some(index))
        }
        None => Ok(None),
    }
}

/// Get index statistics
#[tauri::command]
pub async fn get_index_stats(state: State<'_, AppState>) -> Result<IndexStats, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let cached = database::get_cached_index(&conn).map_err(|e| e.to_string())?;

    match cached {
        Some((data, fetched_at, _)) => {
            let index: AddonIndex =
                serde_json::from_str(&data).map_err(|e| format!("Failed to parse index: {}", e))?;

            // Count categories
            let mut categories = std::collections::HashMap::new();
            for addon in &index.addons {
                *categories.entry(addon.category.clone()).or_insert(0) += 1;
            }

            Ok(IndexStats {
                total_addons: index.addons.len(),
                categories: categories.into_iter().collect(),
                fetched_at,
            })
        }
        None => Ok(IndexStats {
            total_addons: 0,
            categories: Vec::new(),
            fetched_at: String::new(),
        }),
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStats {
    pub total_addons: usize,
    pub categories: Vec<(String, usize)>,
    pub fetched_at: String,
}
