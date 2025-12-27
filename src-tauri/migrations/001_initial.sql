-- Installed addons tracking
CREATE TABLE IF NOT EXISTS installed_addons (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    installed_version TEXT NOT NULL,
    source_type TEXT NOT NULL,  -- 'index' | 'github' | 'local'
    source_repo TEXT,           -- GitHub repo if applicable
    installed_at TEXT NOT NULL, -- ISO 8601 timestamp
    updated_at TEXT NOT NULL,
    auto_update INTEGER DEFAULT 1,
    manifest_path TEXT NOT NULL
);

-- Custom GitHub repositories (not in index)
CREATE TABLE IF NOT EXISTS custom_repos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repo TEXT UNIQUE NOT NULL,       -- owner/repo
    branch TEXT DEFAULT 'main',
    release_type TEXT DEFAULT 'release',
    added_at TEXT NOT NULL,
    last_checked TEXT
);

-- Index cache
CREATE TABLE IF NOT EXISTS index_cache (
    id INTEGER PRIMARY KEY,
    data TEXT NOT NULL,              -- Full JSON
    fetched_at TEXT NOT NULL,
    etag TEXT                        -- For conditional requests
);

-- Application settings
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Download history/queue
CREATE TABLE IF NOT EXISTS downloads (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    addon_slug TEXT NOT NULL,
    version TEXT NOT NULL,
    status TEXT NOT NULL,            -- 'pending' | 'downloading' | 'extracting' | 'complete' | 'failed'
    progress REAL DEFAULT 0,
    error_message TEXT,
    started_at TEXT,
    completed_at TEXT
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_installed_slug ON installed_addons(slug);
CREATE INDEX IF NOT EXISTS idx_downloads_status ON downloads(status);
