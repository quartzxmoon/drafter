-- Initial database schema for PA eDocket Desktop
-- SQLite database for local caching and offline functionality

-- Search cache table
CREATE TABLE IF NOT EXISTS search_cache (
    id TEXT PRIMARY KEY,
    query_hash TEXT NOT NULL,
    results TEXT NOT NULL, -- JSON blob
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL
);

-- Docket cache table
CREATE TABLE IF NOT EXISTS docket_cache (
    id TEXT PRIMARY KEY,
    docket_number TEXT NOT NULL,
    court_id TEXT NOT NULL,
    data TEXT NOT NULL, -- JSON blob
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_updated DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- User watchlists
CREATE TABLE IF NOT EXISTS watchlists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Watchlist items
CREATE TABLE IF NOT EXISTS watchlist_items (
    id TEXT PRIMARY KEY,
    watchlist_id TEXT NOT NULL,
    docket_number TEXT NOT NULL,
    court_id TEXT NOT NULL,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (watchlist_id) REFERENCES watchlists(id) ON DELETE CASCADE
);

-- Export history
CREATE TABLE IF NOT EXISTS export_history (
    id TEXT PRIMARY KEY,
    export_type TEXT NOT NULL,
    file_path TEXT NOT NULL,
    manifest TEXT, -- JSON blob
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Application settings
CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_search_cache_query_hash ON search_cache(query_hash);
CREATE INDEX IF NOT EXISTS idx_search_cache_expires_at ON search_cache(expires_at);
CREATE INDEX IF NOT EXISTS idx_docket_cache_docket_number ON docket_cache(docket_number);
CREATE INDEX IF NOT EXISTS idx_docket_cache_court_id ON docket_cache(court_id);
CREATE INDEX IF NOT EXISTS idx_watchlist_items_watchlist_id ON watchlist_items(watchlist_id);
CREATE INDEX IF NOT EXISTS idx_watchlist_items_docket_number ON watchlist_items(docket_number);
CREATE INDEX IF NOT EXISTS idx_export_history_created_at ON export_history(created_at);
