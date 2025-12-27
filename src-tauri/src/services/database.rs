use crate::error::{AppError, Result};
use crate::models::{CustomRepo, InstalledAddon, ReleaseType, SourceType};
use crate::utils::paths::get_database_path;
use chrono::Utc;
use rusqlite::{params, Connection};
use std::fs;

/// Initialize the database connection and run migrations
pub fn init_database() -> Result<Connection> {
    let db_path = get_database_path().ok_or(AppError::Custom(
        "Could not determine database path".into(),
    ))?;

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(&db_path)?;
    run_migrations(&conn)?;
    Ok(conn)
}

/// Run database migrations
fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(include_str!("../../migrations/001_initial.sql"))?;
    Ok(())
}

// ============================================================================
// Installed Addons
// ============================================================================

/// Get all installed addons
pub fn get_all_installed(conn: &Connection) -> Result<Vec<InstalledAddon>> {
    let mut stmt = conn.prepare(
        "SELECT id, slug, name, installed_version, source_type, source_repo,
                installed_at, updated_at, auto_update, manifest_path
         FROM installed_addons
         ORDER BY name ASC",
    )?;

    let addons = stmt
        .query_map([], |row| {
            Ok(InstalledAddon {
                id: row.get(0)?,
                slug: row.get(1)?,
                name: row.get(2)?,
                installed_version: row.get(3)?,
                source_type: row
                    .get::<_, String>(4)?
                    .parse()
                    .unwrap_or(SourceType::Local),
                source_repo: row.get(5)?,
                installed_at: row.get(6)?,
                updated_at: row.get(7)?,
                auto_update: row.get(8)?,
                manifest_path: row.get(9)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(addons)
}

/// Get an installed addon by slug
pub fn get_installed_by_slug(conn: &Connection, slug: &str) -> Result<Option<InstalledAddon>> {
    let mut stmt = conn.prepare(
        "SELECT id, slug, name, installed_version, source_type, source_repo,
                installed_at, updated_at, auto_update, manifest_path
         FROM installed_addons
         WHERE slug = ?1",
    )?;

    let addon = stmt
        .query_row([slug], |row| {
            Ok(InstalledAddon {
                id: row.get(0)?,
                slug: row.get(1)?,
                name: row.get(2)?,
                installed_version: row.get(3)?,
                source_type: row
                    .get::<_, String>(4)?
                    .parse()
                    .unwrap_or(SourceType::Local),
                source_repo: row.get(5)?,
                installed_at: row.get(6)?,
                updated_at: row.get(7)?,
                auto_update: row.get(8)?,
                manifest_path: row.get(9)?,
            })
        })
        .optional()?;

    Ok(addon)
}

/// Insert a new installed addon
pub fn insert_installed(
    conn: &Connection,
    slug: &str,
    name: &str,
    version: &str,
    source_type: SourceType,
    source_repo: Option<&str>,
    manifest_path: &str,
) -> Result<InstalledAddon> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO installed_addons (slug, name, installed_version, source_type, source_repo, installed_at, updated_at, manifest_path)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(slug) DO UPDATE SET
             installed_version = excluded.installed_version,
             updated_at = excluded.updated_at",
        params![
            slug,
            name,
            version,
            source_type.to_string(),
            source_repo,
            &now,
            &now,
            manifest_path
        ],
    )?;

    get_installed_by_slug(conn, slug)?.ok_or(AppError::AddonNotFound(slug.into()))
}

/// Delete an installed addon
pub fn delete_installed(conn: &Connection, slug: &str) -> Result<()> {
    conn.execute("DELETE FROM installed_addons WHERE slug = ?1", [slug])?;
    Ok(())
}

// ============================================================================
// Custom Repositories
// ============================================================================

/// Get all custom repositories
pub fn get_all_custom_repos(conn: &Connection) -> Result<Vec<CustomRepo>> {
    let mut stmt = conn.prepare(
        "SELECT id, repo, branch, release_type, added_at, last_checked
         FROM custom_repos
         ORDER BY repo ASC",
    )?;

    let repos = stmt
        .query_map([], |row| {
            Ok(CustomRepo {
                id: row.get(0)?,
                repo: row.get(1)?,
                branch: row.get(2)?,
                release_type: row
                    .get::<_, String>(3)?
                    .parse()
                    .unwrap_or(ReleaseType::Release),
                added_at: row.get(4)?,
                last_checked: row.get(5)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(repos)
}

/// Insert a custom repository
pub fn insert_custom_repo(
    conn: &Connection,
    repo: &str,
    branch: &str,
    release_type: ReleaseType,
) -> Result<CustomRepo> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO custom_repos (repo, branch, release_type, added_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(repo) DO UPDATE SET
             branch = excluded.branch,
             release_type = excluded.release_type",
        params![repo, branch, release_type.to_string(), &now],
    )?;

    let mut stmt = conn.prepare(
        "SELECT id, repo, branch, release_type, added_at, last_checked
         FROM custom_repos WHERE repo = ?1",
    )?;

    stmt.query_row([repo], |row| {
        Ok(CustomRepo {
            id: row.get(0)?,
            repo: row.get(1)?,
            branch: row.get(2)?,
            release_type: row
                .get::<_, String>(3)?
                .parse()
                .unwrap_or(ReleaseType::Release),
            added_at: row.get(4)?,
            last_checked: row.get(5)?,
        })
    })
    .map_err(|e| e.into())
}

/// Delete a custom repository
pub fn delete_custom_repo(conn: &Connection, repo: &str) -> Result<()> {
    conn.execute("DELETE FROM custom_repos WHERE repo = ?1", [repo])?;
    Ok(())
}

// ============================================================================
// Index Cache
// ============================================================================

/// Get the cached index data
pub fn get_cached_index(conn: &Connection) -> Result<Option<(String, String, Option<String>)>> {
    let mut stmt = conn.prepare("SELECT data, fetched_at, etag FROM index_cache WHERE id = 1")?;

    let result = stmt
        .query_row([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .optional()?;

    Ok(result)
}

/// Update the cached index data
pub fn update_cached_index(conn: &Connection, data: &str, etag: Option<&str>) -> Result<()> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO index_cache (id, data, fetched_at, etag)
         VALUES (1, ?1, ?2, ?3)
         ON CONFLICT(id) DO UPDATE SET
             data = excluded.data,
             fetched_at = excluded.fetched_at,
             etag = excluded.etag",
        params![data, &now, etag],
    )?;

    Ok(())
}

// ============================================================================
// Settings
// ============================================================================

/// Get a setting value
pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let value = stmt.query_row([key], |row| row.get(0)).optional()?;
    Ok(value)
}

/// Set a setting value
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

// Trait extension for optional query results
trait OptionalExt<T> {
    fn optional(self) -> std::result::Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for std::result::Result<T, rusqlite::Error> {
    fn optional(self) -> std::result::Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
