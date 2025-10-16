-- Bulk Data Import System
-- Migration 005: Local cache for case law, statutes, and automated sync

-- Case law database (from CourtListener)
CREATE TABLE IF NOT EXISTS case_law (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    opinion_id INTEGER UNIQUE NOT NULL,
    cluster_id INTEGER,
    case_name TEXT NOT NULL,
    case_name_short TEXT,
    court TEXT NOT NULL,
    court_id TEXT,
    date_filed TEXT NOT NULL,
    date_filed_year INTEGER,

    -- Citations
    federal_cite_one TEXT,
    federal_cite_two TEXT,
    state_cite_one TEXT,
    state_cite_regional TEXT,
    neutral_cite TEXT,
    westlaw_cite TEXT,
    lexis_cite TEXT,

    -- Content
    plain_text TEXT,
    html TEXT,

    -- Metadata
    precedential_status TEXT DEFAULT 'Precedential',
    citation_count INTEGER DEFAULT 0,
    docket_number TEXT,
    judge_names TEXT, -- JSON array

    -- Full text search
    full_text_indexed TEXT,

    -- Sync metadata
    source TEXT DEFAULT 'courtlistener',
    last_updated TEXT NOT NULL,
    created_at TEXT NOT NULL,

    -- Indexes for fast search
    UNIQUE(opinion_id)
);

-- Statutes database (from GovInfo)
CREATE TABLE IF NOT EXISTS statutes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    package_id TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    collection TEXT NOT NULL, -- USCODE, CFR, etc.

    -- USC specific
    usc_title INTEGER,
    usc_section TEXT,

    -- CFR specific
    cfr_title INTEGER,
    cfr_part TEXT,
    cfr_section TEXT,

    -- Content
    text_content TEXT,
    html_content TEXT,

    -- Citation
    citation TEXT NOT NULL,

    -- Metadata
    date_issued TEXT,
    effective_date TEXT,
    version TEXT,

    -- Download links
    pdf_url TEXT,
    xml_url TEXT,

    -- Sync metadata
    source TEXT DEFAULT 'govinfo',
    last_updated TEXT NOT NULL,
    created_at TEXT NOT NULL,

    UNIQUE(package_id)
);

-- Pennsylvania statutes (from PA legislative sources)
CREATE TABLE IF NOT EXISTS pa_statutes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title INTEGER NOT NULL,
    chapter TEXT,
    section TEXT NOT NULL,
    subsection TEXT,

    -- Content
    heading TEXT,
    text_content TEXT NOT NULL,

    -- Citation
    citation TEXT NOT NULL,

    -- Metadata
    effective_date TEXT,
    history TEXT,

    -- Sync metadata
    last_updated TEXT NOT NULL,
    created_at TEXT NOT NULL,

    UNIQUE(title, section)
);

-- Court rules
CREATE TABLE IF NOT EXISTS court_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_set TEXT NOT NULL, -- FRCP, FRAP, Pa.R.Civ.P., etc.
    rule_number TEXT NOT NULL,

    -- Content
    title TEXT NOT NULL,
    text_content TEXT NOT NULL,

    -- Citation
    citation TEXT NOT NULL,

    -- Metadata
    effective_date TEXT,
    last_amended TEXT,

    -- Sync metadata
    last_updated TEXT NOT NULL,
    created_at TEXT NOT NULL,

    UNIQUE(rule_set, rule_number)
);

-- Data sync jobs and status
CREATE TABLE IF NOT EXISTS sync_jobs (
    id TEXT PRIMARY KEY,
    job_type TEXT NOT NULL, -- bulk_initial, incremental, on_demand
    data_source TEXT NOT NULL, -- courtlistener, govinfo, pa_legislative
    collection TEXT, -- USCODE, CFR, opinions, etc.

    -- Job parameters
    start_date TEXT,
    end_date TEXT,
    filters TEXT, -- JSON

    -- Progress tracking
    status TEXT DEFAULT 'pending', -- pending, running, completed, failed, paused
    total_items INTEGER DEFAULT 0,
    processed_items INTEGER DEFAULT 0,
    failed_items INTEGER DEFAULT 0,

    -- Results
    error_message TEXT,
    result_summary TEXT, -- JSON

    -- Timing
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL,

    -- Resume capability
    last_checkpoint TEXT, -- JSON with resume info
    resume_token TEXT
);

-- Sync schedule
CREATE TABLE IF NOT EXISTS sync_schedules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    data_source TEXT NOT NULL,
    collection TEXT,

    -- Schedule (cron-like)
    schedule_type TEXT NOT NULL, -- daily, weekly, monthly, custom_cron
    schedule_value TEXT, -- cron expression or specific time

    -- Configuration
    enabled BOOLEAN DEFAULT 1,
    filters TEXT, -- JSON
    incremental BOOLEAN DEFAULT 1, -- Only fetch new/updated

    -- Last run
    last_run_at TEXT,
    last_run_status TEXT,
    next_run_at TEXT,

    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Downloaded files tracking
CREATE TABLE IF NOT EXISTS downloaded_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id TEXT NOT NULL, -- opinion_id, package_id, etc.
    source_type TEXT NOT NULL, -- case_law, statute, etc.
    file_type TEXT NOT NULL, -- pdf, xml, txt
    file_path TEXT NOT NULL,
    file_size INTEGER,
    checksum TEXT,
    downloaded_at TEXT NOT NULL,

    UNIQUE(source_id, file_type)
);

-- API rate limiting tracking
CREATE TABLE IF NOT EXISTS api_rate_limits (
    api_source TEXT PRIMARY KEY,
    requests_made INTEGER DEFAULT 0,
    requests_limit INTEGER NOT NULL,
    window_start TEXT NOT NULL,
    window_duration INTEGER NOT NULL, -- seconds
    last_request_at TEXT
);

-- Create indexes for fast searching
CREATE INDEX IF NOT EXISTS idx_case_law_court ON case_law(court);
CREATE INDEX IF NOT EXISTS idx_case_law_date ON case_law(date_filed);
CREATE INDEX IF NOT EXISTS idx_case_law_year ON case_law(date_filed_year);
CREATE INDEX IF NOT EXISTS idx_case_law_cite ON case_law(federal_cite_one, state_cite_one);
CREATE INDEX IF NOT EXISTS idx_case_law_docket ON case_law(docket_number);

CREATE INDEX IF NOT EXISTS idx_statutes_collection ON statutes(collection);
CREATE INDEX IF NOT EXISTS idx_statutes_usc ON statutes(usc_title, usc_section);
CREATE INDEX IF NOT EXISTS idx_statutes_cfr ON statutes(cfr_title, cfr_part);

CREATE INDEX IF NOT EXISTS idx_pa_statutes_title ON pa_statutes(title, section);

CREATE INDEX IF NOT EXISTS idx_sync_jobs_status ON sync_jobs(status);
CREATE INDEX IF NOT EXISTS idx_sync_jobs_source ON sync_jobs(data_source);

-- Full-text search virtual table for case law
CREATE VIRTUAL TABLE IF NOT EXISTS case_law_fts USING fts5(
    case_name,
    plain_text,
    content='case_law',
    content_rowid='id'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS case_law_fts_insert AFTER INSERT ON case_law BEGIN
    INSERT INTO case_law_fts(rowid, case_name, plain_text)
    VALUES (new.id, new.case_name, new.plain_text);
END;

CREATE TRIGGER IF NOT EXISTS case_law_fts_delete AFTER DELETE ON case_law BEGIN
    DELETE FROM case_law_fts WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS case_law_fts_update AFTER UPDATE ON case_law BEGIN
    DELETE FROM case_law_fts WHERE rowid = old.id;
    INSERT INTO case_law_fts(rowid, case_name, plain_text)
    VALUES (new.id, new.case_name, new.plain_text);
END;

-- Insert default sync schedules
INSERT OR IGNORE INTO sync_schedules (id, name, data_source, collection, schedule_type, schedule_value, enabled, incremental, created_at, updated_at)
VALUES
    ('sync_cl_pa_supreme', 'PA Supreme Court - Daily', 'courtlistener', 'pa', 'daily', '02:00', 1, 1, datetime('now'), datetime('now')),
    ('sync_cl_pa_superior', 'PA Superior Court - Daily', 'courtlistener', 'pasuperct', 'daily', '03:00', 1, 1, datetime('now'), datetime('now')),
    ('sync_cl_pa_commw', 'PA Commonwealth Court - Daily', 'courtlistener', 'pacommwct', 'daily', '04:00', 1, 1, datetime('now'), datetime('now')),
    ('sync_cl_3rd_circuit', '3rd Circuit - Daily', 'courtlistener', 'ca3', 'daily', '05:00', 1, 1, datetime('now'), datetime('now')),
    ('sync_govinfo_uscode', 'US Code - Weekly', 'govinfo', 'USCODE', 'weekly', 'Sunday 01:00', 1, 1, datetime('now'), datetime('now')),
    ('sync_govinfo_cfr', 'CFR - Weekly', 'govinfo', 'CFR', 'weekly', 'Sunday 02:00', 1, 1, datetime('now'), datetime('now'));

-- Insert rate limit configuration
INSERT OR IGNORE INTO api_rate_limits (api_source, requests_limit, window_duration, window_start, last_request_at)
VALUES
    ('courtlistener', 300, 60, datetime('now'), datetime('now')), -- 5 per second = 300 per minute
    ('govinfo', 600, 60, datetime('now'), datetime('now')); -- 10 per second = 600 per minute
