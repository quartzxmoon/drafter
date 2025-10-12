-- PA eDocket Desktop - Initial Database Schema
-- Production-grade schema for legal document management and search

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "btree_gin";

-- Sources table for tracking data providers
CREATE TABLE IF NOT EXISTS sources (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    base_url TEXT,
    api_version TEXT,
    rate_limit INTEGER DEFAULT 30,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert default sources
INSERT INTO sources (name, description, base_url, api_version, rate_limit) VALUES 
    ('courtlistener', 'CourtListener API for legal opinions and dockets', 'https://www.courtlistener.com/api/rest/v3/', 'v3', 5),
    ('govinfo', 'GovInfo API for government documents', 'https://api.govinfo.gov/', 'v1', 10),
    ('ujs_portal', 'PA UJS Portal for court dockets', 'https://ujsportal.pacourts.us/', 'web', 30),
    ('pacfile', 'PA E-Filing System', 'https://www.pacourts.us/pacfile/', 'web', 10)
ON CONFLICT (name) DO NOTHING;

-- Main documents table
CREATE TABLE IF NOT EXISTS documents (
    id BIGSERIAL PRIMARY KEY,
    source_id INTEGER REFERENCES sources(id) NOT NULL,
    external_id TEXT NOT NULL,
    type TEXT NOT NULL CHECK (type IN ('opinion', 'docket', 'filing', 'rule', 'audio', 'order', 'motion', 'brief')),
    court TEXT,
    jurisdiction TEXT,
    docket_number TEXT,
    case_name TEXT,
    date_filed TIMESTAMPTZ,
    date_modified TIMESTAMPTZ,
    cites JSONB DEFAULT '[]'::jsonb,
    parties JSONB DEFAULT '[]'::jsonb,
    judges JSONB DEFAULT '[]'::jsonb,
    attorneys JSONB DEFAULT '[]'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    text_summary TEXT,
    full_text TEXT,
    sha256 CHAR(64) NOT NULL,
    url TEXT NOT NULL,
    pdf_path TEXT,
    txt_path TEXT,
    file_size BIGINT,
    page_count INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(source_id, external_id)
);

-- Ingest state tracking for incremental updates
CREATE TABLE IF NOT EXISTS ingest_state (
    id SERIAL PRIMARY KEY,
    source TEXT NOT NULL,
    collection TEXT NOT NULL,
    last_success_timestamp TIMESTAMPTZ,
    last_attempt_timestamp TIMESTAMPTZ,
    cursor TEXT,
    status TEXT DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'success', 'error')),
    error_message TEXT,
    records_processed INTEGER DEFAULT 0,
    records_failed INTEGER DEFAULT 0,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(source, collection)
);

-- Search cache for performance
CREATE TABLE IF NOT EXISTS search_cache (
    id BIGSERIAL PRIMARY KEY,
    query_hash CHAR(64) NOT NULL UNIQUE,
    query_params JSONB NOT NULL,
    results JSONB NOT NULL,
    result_count INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    hit_count INTEGER DEFAULT 1
);

-- User watchlists for case tracking
CREATE TABLE IF NOT EXISTS watchlists (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    description TEXT,
    user_id TEXT, -- For future multi-user support
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS watchlist_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    watchlist_id UUID REFERENCES watchlists(id) ON DELETE CASCADE,
    document_id BIGINT REFERENCES documents(id) ON DELETE CASCADE,
    docket_number TEXT,
    court TEXT,
    notes TEXT,
    alert_enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(watchlist_id, document_id)
);

-- Export history tracking
CREATE TABLE IF NOT EXISTS export_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    export_type TEXT NOT NULL CHECK (export_type IN ('json', 'csv', 'pdf', 'zip')),
    query_params JSONB,
    file_path TEXT,
    file_size BIGINT,
    record_count INTEGER,
    status TEXT DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'failed')),
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- Application settings
CREATE TABLE IF NOT EXISTS settings (
    id SERIAL PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    value JSONB NOT NULL,
    description TEXT,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert default settings
INSERT INTO settings (key, value, description) VALUES
    ('search_cache_ttl', '3600', 'Search cache TTL in seconds'),
    ('max_export_records', '10000', 'Maximum records per export'),
    ('default_page_size', '25', 'Default pagination size'),
    ('rate_limit_window', '60', 'Rate limit window in seconds')
ON CONFLICT (key) DO NOTHING;

-- Job queue for background processing
CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    type TEXT NOT NULL,
    payload JSONB NOT NULL,
    status TEXT DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    priority INTEGER DEFAULT 0,
    attempts INTEGER DEFAULT 0,
    max_attempts INTEGER DEFAULT 3,
    scheduled_at TIMESTAMPTZ DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_documents_source_external ON documents(source_id, external_id);
CREATE INDEX IF NOT EXISTS idx_documents_type ON documents(type);
CREATE INDEX IF NOT EXISTS idx_documents_court ON documents(court);
CREATE INDEX IF NOT EXISTS idx_documents_jurisdiction ON documents(jurisdiction);
CREATE INDEX IF NOT EXISTS idx_documents_docket_number ON documents(docket_number);
CREATE INDEX IF NOT EXISTS idx_documents_date_filed ON documents(date_filed);
CREATE INDEX IF NOT EXISTS idx_documents_date_modified ON documents(date_modified);
CREATE INDEX IF NOT EXISTS idx_documents_sha256 ON documents(sha256);
CREATE INDEX IF NOT EXISTS idx_documents_full_text_gin ON documents USING gin(to_tsvector('english', full_text));
CREATE INDEX IF NOT EXISTS idx_documents_case_name_gin ON documents USING gin(to_tsvector('english', case_name));
CREATE INDEX IF NOT EXISTS idx_documents_metadata_gin ON documents USING gin(metadata);
CREATE INDEX IF NOT EXISTS idx_documents_cites_gin ON documents USING gin(cites);
CREATE INDEX IF NOT EXISTS idx_documents_parties_gin ON documents USING gin(parties);

CREATE INDEX IF NOT EXISTS idx_search_cache_expires ON search_cache(expires_at);
CREATE INDEX IF NOT EXISTS idx_search_cache_created ON search_cache(created_at);

CREATE INDEX IF NOT EXISTS idx_watchlist_items_watchlist ON watchlist_items(watchlist_id);
CREATE INDEX IF NOT EXISTS idx_watchlist_items_document ON watchlist_items(document_id);
CREATE INDEX IF NOT EXISTS idx_watchlist_items_docket ON watchlist_items(docket_number);

CREATE INDEX IF NOT EXISTS idx_export_history_status ON export_history(status);
CREATE INDEX IF NOT EXISTS idx_export_history_created ON export_history(created_at);

CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
CREATE INDEX IF NOT EXISTS idx_jobs_type ON jobs(type);
CREATE INDEX IF NOT EXISTS idx_jobs_scheduled ON jobs(scheduled_at);
CREATE INDEX IF NOT EXISTS idx_jobs_priority ON jobs(priority DESC, created_at);

CREATE INDEX IF NOT EXISTS idx_ingest_state_source_collection ON ingest_state(source, collection);

-- Triggers for updated_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_sources_updated_at BEFORE UPDATE ON sources
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_documents_updated_at BEFORE UPDATE ON documents
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ingest_state_updated_at BEFORE UPDATE ON ingest_state
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_watchlists_updated_at BEFORE UPDATE ON watchlists
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_settings_updated_at BEFORE UPDATE ON settings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Views for common queries
CREATE OR REPLACE VIEW recent_documents AS
SELECT 
    d.*,
    s.name as source_name
FROM documents d
JOIN sources s ON d.source_id = s.id
WHERE d.created_at >= NOW() - INTERVAL '30 days'
ORDER BY d.created_at DESC;

CREATE OR REPLACE VIEW document_stats AS
SELECT 
    s.name as source,
    d.type,
    d.court,
    COUNT(*) as document_count,
    MIN(d.date_filed) as earliest_date,
    MAX(d.date_filed) as latest_date,
    AVG(d.file_size) as avg_file_size
FROM documents d
JOIN sources s ON d.source_id = s.id
GROUP BY s.name, d.type, d.court;

-- Comments for documentation
COMMENT ON TABLE sources IS 'Data source providers (CourtListener, GovInfo, etc.)';
COMMENT ON TABLE documents IS 'Main document storage with full metadata and text';
COMMENT ON TABLE ingest_state IS 'Tracks incremental ingestion progress per source/collection';
COMMENT ON TABLE search_cache IS 'Caches search results for performance';
COMMENT ON TABLE watchlists IS 'User-defined case watchlists';
COMMENT ON TABLE watchlist_items IS 'Individual items in watchlists';
COMMENT ON TABLE export_history IS 'Tracks export operations and files';
COMMENT ON TABLE settings IS 'Application configuration settings';
COMMENT ON TABLE jobs IS 'Background job queue for processing';

-- Grant permissions (adjust as needed for your deployment)
-- GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO pa_edocket_app;
-- GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO pa_edocket_app;
