-- Hierarchical Case Organization System
-- Migration 004: Add case hierarchy and relationships

-- Case folders/categories for organization
CREATE TABLE IF NOT EXISTS case_folders (
    id TEXT PRIMARY KEY,
    parent_folder_id TEXT,
    name TEXT NOT NULL,
    description TEXT,
    color TEXT DEFAULT '#3B82F6',
    icon TEXT DEFAULT 'folder',
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (parent_folder_id) REFERENCES case_folders(id) ON DELETE CASCADE
);

-- Link matters to folders (many-to-many)
CREATE TABLE IF NOT EXISTS matter_folders (
    matter_id TEXT NOT NULL,
    folder_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (matter_id, folder_id),
    FOREIGN KEY (matter_id) REFERENCES matters(id) ON DELETE CASCADE,
    FOREIGN KEY (folder_id) REFERENCES case_folders(id) ON DELETE CASCADE
);

-- Related/linked cases
CREATE TABLE IF NOT EXISTS related_matters (
    matter_id TEXT NOT NULL,
    related_matter_id TEXT NOT NULL,
    relationship_type TEXT NOT NULL, -- consolidated, related, predecessor, successor, companion
    description TEXT,
    created_at TEXT NOT NULL,
    PRIMARY KEY (matter_id, related_matter_id),
    FOREIGN KEY (matter_id) REFERENCES matters(id) ON DELETE CASCADE,
    FOREIGN KEY (related_matter_id) REFERENCES matters(id) ON DELETE CASCADE
);

-- Matter tags for flexible categorization
CREATE TABLE IF NOT EXISTS matter_tags (
    id TEXT PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    color TEXT DEFAULT '#6B7280',
    description TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS matter_tag_assignments (
    matter_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (matter_id, tag_id),
    FOREIGN KEY (matter_id) REFERENCES matters(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES matter_tags(id) ON DELETE CASCADE
);

-- Practice areas for hierarchical organization
CREATE TABLE IF NOT EXISTS practice_areas (
    id TEXT PRIMARY KEY,
    parent_area_id TEXT,
    name TEXT NOT NULL,
    description TEXT,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    FOREIGN KEY (parent_area_id) REFERENCES practice_areas(id) ON DELETE CASCADE
);

-- Link matters to practice areas
CREATE TABLE IF NOT EXISTS matter_practice_areas (
    matter_id TEXT NOT NULL,
    practice_area_id TEXT NOT NULL,
    is_primary BOOLEAN DEFAULT 0,
    created_at TEXT NOT NULL,
    PRIMARY KEY (matter_id, practice_area_id),
    FOREIGN KEY (matter_id) REFERENCES matters(id) ON DELETE CASCADE,
    FOREIGN KEY (practice_area_id) REFERENCES practice_areas(id) ON DELETE CASCADE
);

-- Case status tracking with history
CREATE TABLE IF NOT EXISTS matter_status_history (
    id TEXT PRIMARY KEY,
    matter_id TEXT NOT NULL,
    old_status TEXT,
    new_status TEXT NOT NULL,
    reason TEXT,
    changed_by TEXT,
    changed_at TEXT NOT NULL,
    FOREIGN KEY (matter_id) REFERENCES matters(id) ON DELETE CASCADE
);

-- Create indexes for hierarchical queries
CREATE INDEX IF NOT EXISTS idx_case_folders_parent ON case_folders(parent_folder_id);
CREATE INDEX IF NOT EXISTS idx_matter_folders_folder ON matter_folders(folder_id);
CREATE INDEX IF NOT EXISTS idx_related_matters_related ON related_matters(related_matter_id);
CREATE INDEX IF NOT EXISTS idx_practice_areas_parent ON practice_areas(parent_area_id);
CREATE INDEX IF NOT EXISTS idx_matter_practice_areas_area ON matter_practice_areas(practice_area_id);

-- Insert default practice areas with hierarchy
INSERT INTO practice_areas (id, parent_area_id, name, description, sort_order, created_at) VALUES
('pa_civil', NULL, 'Civil Litigation', 'All civil litigation matters', 1, datetime('now')),
('pa_civil_pi', 'pa_civil', 'Personal Injury', 'Personal injury and wrongful death cases', 1, datetime('now')),
('pa_civil_contract', 'pa_civil', 'Contract Disputes', 'Breach of contract and commercial disputes', 2, datetime('now')),
('pa_civil_property', 'pa_civil', 'Property', 'Real estate and property disputes', 3, datetime('now')),
('pa_criminal', NULL, 'Criminal Defense', 'Criminal defense matters', 2, datetime('now')),
('pa_criminal_felony', 'pa_criminal', 'Felony', 'Felony criminal cases', 1, datetime('now')),
('pa_criminal_misdemeanor', 'pa_criminal', 'Misdemeanor', 'Misdemeanor criminal cases', 2, datetime('now')),
('pa_criminal_dui', 'pa_criminal', 'DUI/DWI', 'Driving under influence cases', 3, datetime('now')),
('pa_family', NULL, 'Family Law', 'Family law matters', 3, datetime('now')),
('pa_family_divorce', 'pa_family', 'Divorce', 'Divorce and dissolution', 1, datetime('now')),
('pa_family_custody', 'pa_family', 'Custody', 'Child custody and support', 2, datetime('now')),
('pa_family_pfa', 'pa_family', 'PFA', 'Protection from abuse orders', 3, datetime('now')),
('pa_estate', NULL, 'Estate Planning', 'Wills, trusts, and probate', 4, datetime('now')),
('pa_business', NULL, 'Business Law', 'Corporate and business matters', 5, datetime('now')),
('pa_employment', NULL, 'Employment Law', 'Employment and labor disputes', 6, datetime('now')),
('pa_immigration', NULL, 'Immigration', 'Immigration matters', 7, datetime('now')),
('pa_bankruptcy', NULL, 'Bankruptcy', 'Bankruptcy proceedings', 8, datetime('now'));

-- Insert default case folders
INSERT INTO case_folders (id, parent_folder_id, name, description, color, icon, sort_order, created_at, updated_at) VALUES
('folder_active', NULL, 'Active Cases', 'Currently active matters', '#10B981', 'folder-open', 1, datetime('now'), datetime('now')),
('folder_pending', NULL, 'Pending Cases', 'Cases pending initiation or response', '#F59E0B', 'clock', 2, datetime('now'), datetime('now')),
('folder_trial', NULL, 'Trial Prep', 'Cases in trial preparation', '#EF4444', 'gavel', 3, datetime('now'), datetime('now')),
('folder_settlement', NULL, 'Settlement Discussions', 'Cases in settlement negotiation', '#8B5CF6', 'handshake', 4, datetime('now'), datetime('now')),
('folder_closed', NULL, 'Closed Cases', 'Completed or closed matters', '#6B7280', 'archive', 5, datetime('now'), datetime('now')),
('folder_appeals', NULL, 'Appeals', 'Appellate matters', '#3B82F6', 'arrow-up-circle', 6, datetime('now'), datetime('now'));
