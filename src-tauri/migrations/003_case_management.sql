-- Case Management System Database Schema
-- Migration 003: Case Management Tables

-- Clients table
CREATE TABLE IF NOT EXISTS clients (
    id TEXT PRIMARY KEY,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    address TEXT,
    city TEXT,
    state TEXT DEFAULT 'PA',
    zip_code TEXT,
    date_of_birth TEXT,
    ssn_encrypted TEXT, -- Encrypted SSN
    notes TEXT,
    client_type TEXT DEFAULT 'individual', -- individual, business, government
    business_name TEXT,
    contact_person TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    status TEXT DEFAULT 'active' -- active, inactive, archived
);

-- Matters/Cases table
CREATE TABLE IF NOT EXISTS matters (
    id TEXT PRIMARY KEY,
    client_id TEXT NOT NULL,
    matter_number TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    matter_type TEXT NOT NULL, -- civil, criminal, family, etc.
    case_type TEXT, -- specific type: divorce, DUI, contract dispute, etc.
    court_level TEXT, -- MDJ, CP, APP
    court_name TEXT,
    county TEXT,
    docket_number TEXT,
    judge_name TEXT,
    opposing_party TEXT,
    opposing_counsel TEXT,
    opposing_counsel_firm TEXT,
    opposing_counsel_email TEXT,
    opposing_counsel_phone TEXT,
    filing_date TEXT,
    status TEXT DEFAULT 'active', -- active, pending, closed, archived
    outcome TEXT,
    settlement_amount REAL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    closed_at TEXT,
    FOREIGN KEY (client_id) REFERENCES clients(id) ON DELETE CASCADE
);

-- Case participants (additional parties beyond client)
CREATE TABLE IF NOT EXISTS case_participants (
    id TEXT PRIMARY KEY,
    matter_id TEXT NOT NULL,
    role TEXT NOT NULL, -- plaintiff, defendant, witness, expert, etc.
    party_type TEXT NOT NULL, -- person, organization
    first_name TEXT,
    last_name TEXT,
    organization_name TEXT,
    email TEXT,
    phone TEXT,
    address TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (matter_id) REFERENCES matters(id) ON DELETE CASCADE
);

-- Case events and timeline
CREATE TABLE IF NOT EXISTS case_events (
    id TEXT PRIMARY KEY,
    matter_id TEXT NOT NULL,
    event_type TEXT NOT NULL, -- filing, hearing, deadline, conference, trial, etc.
    title TEXT NOT NULL,
    description TEXT,
    event_date TEXT NOT NULL,
    event_time TEXT,
    location TEXT,
    participants TEXT, -- JSON array
    outcome TEXT,
    notes TEXT,
    reminder_set BOOLEAN DEFAULT 0,
    reminder_date TEXT,
    completed BOOLEAN DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (matter_id) REFERENCES matters(id) ON DELETE CASCADE
);

-- Tasks and deadlines
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    matter_id TEXT,
    assigned_to TEXT,
    title TEXT NOT NULL,
    description TEXT,
    priority TEXT DEFAULT 'medium', -- low, medium, high, urgent
    due_date TEXT,
    due_time TEXT,
    status TEXT DEFAULT 'pending', -- pending, in_progress, completed, cancelled
    category TEXT, -- research, drafting, filing, client_communication, etc.
    estimated_hours REAL,
    actual_hours REAL,
    completed_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (matter_id) REFERENCES matters(id) ON DELETE SET NULL
);

-- Documents linked to cases
CREATE TABLE IF NOT EXISTS case_documents (
    id TEXT PRIMARY KEY,
    matter_id TEXT NOT NULL,
    document_type TEXT NOT NULL, -- motion, brief, order, evidence, correspondence, etc.
    title TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER,
    mime_type TEXT,
    version INTEGER DEFAULT 1,
    is_template BOOLEAN DEFAULT 0,
    filed_with_court BOOLEAN DEFAULT 0,
    filing_date TEXT,
    created_by TEXT,
    tags TEXT, -- JSON array
    notes TEXT,
    checksum TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (matter_id) REFERENCES matters(id) ON DELETE CASCADE
);

-- Document versions for revision tracking
CREATE TABLE IF NOT EXISTS document_versions (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL,
    version INTEGER NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER,
    changes_summary TEXT,
    created_by TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (document_id) REFERENCES case_documents(id) ON DELETE CASCADE,
    UNIQUE(document_id, version)
);

-- Case notes and journal
CREATE TABLE IF NOT EXISTS case_notes (
    id TEXT PRIMARY KEY,
    matter_id TEXT NOT NULL,
    note_type TEXT DEFAULT 'general', -- general, research, strategy, client_communication
    title TEXT,
    content TEXT NOT NULL,
    is_private BOOLEAN DEFAULT 0,
    tags TEXT, -- JSON array
    created_by TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (matter_id) REFERENCES matters(id) ON DELETE CASCADE
);

-- Time entries for billing
CREATE TABLE IF NOT EXISTS time_entries (
    id TEXT PRIMARY KEY,
    matter_id TEXT NOT NULL,
    task_id TEXT,
    attorney_id TEXT,
    entry_date TEXT NOT NULL,
    hours REAL NOT NULL,
    rate REAL,
    description TEXT NOT NULL,
    billable BOOLEAN DEFAULT 1,
    billed BOOLEAN DEFAULT 0,
    invoice_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (matter_id) REFERENCES matters(id) ON DELETE CASCADE,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE SET NULL
);

-- Expenses
CREATE TABLE IF NOT EXISTS expenses (
    id TEXT PRIMARY KEY,
    matter_id TEXT NOT NULL,
    expense_date TEXT NOT NULL,
    category TEXT NOT NULL, -- filing_fee, service_fee, expert_fee, travel, etc.
    amount REAL NOT NULL,
    description TEXT NOT NULL,
    receipt_path TEXT,
    billable BOOLEAN DEFAULT 1,
    billed BOOLEAN DEFAULT 0,
    invoice_id TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (matter_id) REFERENCES matters(id) ON DELETE CASCADE
);

-- Contact management (broader than just clients)
CREATE TABLE IF NOT EXISTS contacts (
    id TEXT PRIMARY KEY,
    contact_type TEXT NOT NULL, -- attorney, expert, court_staff, vendor, etc.
    first_name TEXT,
    last_name TEXT,
    organization TEXT,
    title TEXT,
    email TEXT,
    phone TEXT,
    mobile TEXT,
    fax TEXT,
    address TEXT,
    city TEXT,
    state TEXT,
    zip_code TEXT,
    bar_number TEXT,
    specialization TEXT,
    notes TEXT,
    tags TEXT, -- JSON array
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Court information
CREATE TABLE IF NOT EXISTS courts (
    id TEXT PRIMARY KEY,
    court_name TEXT NOT NULL,
    court_level TEXT NOT NULL, -- MDJ, CP, APP, Federal
    county TEXT,
    address TEXT,
    city TEXT,
    state TEXT DEFAULT 'PA',
    zip_code TEXT,
    phone TEXT,
    fax TEXT,
    website TEXT,
    efiling_enabled BOOLEAN DEFAULT 0,
    efiling_provider TEXT,
    local_rules_url TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Document templates with variable mapping
CREATE TABLE IF NOT EXISTS document_templates (
    id TEXT PRIMARY KEY,
    template_name TEXT NOT NULL,
    document_type TEXT NOT NULL,
    court_level TEXT,
    matter_types TEXT, -- JSON array of applicable matter types
    description TEXT,
    template_content TEXT NOT NULL, -- Rich text with variables
    variable_schema TEXT NOT NULL, -- JSON schema of required variables
    auto_populate_rules TEXT, -- JSON mapping of case fields to template variables
    formatting_rules TEXT, -- JSON court-specific formatting
    file_path TEXT,
    is_public BOOLEAN DEFAULT 1,
    is_pro_se_friendly BOOLEAN DEFAULT 0,
    category TEXT, -- pleading, motion, discovery, correspondence, etc.
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Auto-generation rules for documents
CREATE TABLE IF NOT EXISTS auto_generation_rules (
    id TEXT PRIMARY KEY,
    template_id TEXT NOT NULL,
    trigger_event TEXT NOT NULL, -- case_filed, hearing_scheduled, deadline_approaching, etc.
    matter_type TEXT,
    conditions TEXT, -- JSON conditions that must be met
    variable_mappings TEXT NOT NULL, -- JSON mapping from case data to template
    priority INTEGER DEFAULT 0,
    enabled BOOLEAN DEFAULT 1,
    created_at TEXT NOT NULL,
    FOREIGN KEY (template_id) REFERENCES document_templates(id) ON DELETE CASCADE
);

-- User settings and preferences
CREATE TABLE IF NOT EXISTS user_settings (
    id TEXT PRIMARY KEY,
    user_id TEXT UNIQUE NOT NULL,
    attorney_name TEXT,
    bar_number TEXT,
    firm_name TEXT,
    firm_address TEXT,
    firm_phone TEXT,
    firm_email TEXT,
    default_signature TEXT,
    letterhead_template TEXT,
    billing_rate REAL,
    timezone TEXT DEFAULT 'America/New_York',
    date_format TEXT DEFAULT 'MM/DD/YYYY',
    pro_se_mode BOOLEAN DEFAULT 0,
    theme TEXT DEFAULT 'professional',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Saved searches and filters
CREATE TABLE IF NOT EXISTS saved_searches (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    search_name TEXT NOT NULL,
    search_type TEXT NOT NULL, -- case_law, statutes, dockets, matters
    search_params TEXT NOT NULL, -- JSON
    created_at TEXT NOT NULL
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_matters_client ON matters(client_id);
CREATE INDEX IF NOT EXISTS idx_matters_status ON matters(status);
CREATE INDEX IF NOT EXISTS idx_matters_docket ON matters(docket_number);
CREATE INDEX IF NOT EXISTS idx_events_matter ON case_events(matter_id);
CREATE INDEX IF NOT EXISTS idx_events_date ON case_events(event_date);
CREATE INDEX IF NOT EXISTS idx_tasks_matter ON tasks(matter_id);
CREATE INDEX IF NOT EXISTS idx_tasks_due ON tasks(due_date);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_documents_matter ON case_documents(matter_id);
CREATE INDEX IF NOT EXISTS idx_documents_type ON case_documents(document_type);
CREATE INDEX IF NOT EXISTS idx_time_entries_matter ON time_entries(matter_id);
CREATE INDEX IF NOT EXISTS idx_expenses_matter ON expenses(matter_id);
CREATE INDEX IF NOT EXISTS idx_participants_matter ON case_participants(matter_id);
CREATE INDEX IF NOT EXISTS idx_notes_matter ON case_notes(matter_id);
