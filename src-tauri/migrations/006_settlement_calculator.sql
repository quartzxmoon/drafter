-- Settlement Calculator Comprehensive Database Schema
-- Supports full settlement calculation, negotiation tracking, and AI analytics

-- ============= SETTLEMENT CALCULATIONS =============

CREATE TABLE IF NOT EXISTS settlement_calculations (
    id TEXT PRIMARY KEY,
    matter_id TEXT NOT NULL,
    case_type TEXT NOT NULL,
    plaintiff_name TEXT NOT NULL,
    defendant_name TEXT NOT NULL,
    incident_date TIMESTAMP,

    -- Damages totals
    total_economic_damages REAL NOT NULL DEFAULT 0.0,
    total_non_economic_damages REAL NOT NULL DEFAULT 0.0,
    total_punitive_damages REAL DEFAULT NULL,
    total_damages REAL NOT NULL DEFAULT 0.0,

    -- Settlement recommendations
    recommended_demand REAL NOT NULL,
    minimum_settlement REAL NOT NULL,
    target_settlement REAL NOT NULL,

    -- Jurisdiction
    jurisdiction TEXT NOT NULL,
    state_code TEXT NOT NULL,
    adjusted_for_caps BOOLEAN NOT NULL DEFAULT FALSE,

    -- Attorney fees
    estimated_attorney_fees REAL NOT NULL DEFAULT 0.0,
    litigation_costs_to_date REAL NOT NULL DEFAULT 0.0,
    projected_additional_costs REAL NOT NULL DEFAULT 0.0,
    net_to_client REAL NOT NULL DEFAULT 0.0,

    -- Negotiation tracking
    current_negotiation_round INTEGER NOT NULL DEFAULT 0,

    -- Metadata
    calculated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    calculated_by TEXT NOT NULL,
    last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    version TEXT NOT NULL DEFAULT '2.0.0',

    FOREIGN KEY (matter_id) REFERENCES cases(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_settlement_matter ON settlement_calculations(matter_id);
CREATE INDEX IF NOT EXISTS idx_settlement_date ON settlement_calculations(calculated_at);
CREATE INDEX IF NOT EXISTS idx_settlement_jurisdiction ON settlement_calculations(jurisdiction);

-- ============= ECONOMIC DAMAGES =============

CREATE TABLE IF NOT EXISTS economic_damages (
    id TEXT PRIMARY KEY,
    settlement_calculation_id TEXT NOT NULL,

    -- Medical expenses
    past_medical_expenses REAL NOT NULL DEFAULT 0.0,
    future_medical_expenses REAL NOT NULL DEFAULT 0.0,

    -- Lost income
    past_lost_wages REAL NOT NULL DEFAULT 0.0,
    future_lost_earning_capacity REAL NOT NULL DEFAULT 0.0,
    lost_benefits REAL NOT NULL DEFAULT 0.0,

    -- Property damage
    property_damage REAL NOT NULL DEFAULT 0.0,

    -- Other economic losses
    rehabilitation_costs REAL NOT NULL DEFAULT 0.0,
    home_modification_costs REAL NOT NULL DEFAULT 0.0,
    assistive_device_costs REAL NOT NULL DEFAULT 0.0,
    transportation_costs REAL NOT NULL DEFAULT 0.0,
    other_expenses REAL NOT NULL DEFAULT 0.0,

    -- Totals
    total_past_economic REAL NOT NULL DEFAULT 0.0,
    total_future_economic REAL NOT NULL DEFAULT 0.0,
    total_economic REAL NOT NULL DEFAULT 0.0,

    -- Present value calculations
    discount_rate REAL NOT NULL DEFAULT 0.03,
    present_value_future_damages REAL NOT NULL DEFAULT 0.0,

    FOREIGN KEY (settlement_calculation_id) REFERENCES settlement_calculations(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_economic_damages_calc ON economic_damages(settlement_calculation_id);

-- ============= MEDICAL EXPENSES DETAIL =============

CREATE TABLE IF NOT EXISTS medical_expenses (
    id TEXT PRIMARY KEY,
    economic_damages_id TEXT NOT NULL,

    date TIMESTAMP NOT NULL,
    provider TEXT NOT NULL,
    description TEXT NOT NULL,
    amount REAL NOT NULL,
    category TEXT NOT NULL, -- Emergency, Hospital, Surgery, etc.
    is_future BOOLEAN NOT NULL DEFAULT FALSE,

    FOREIGN KEY (economic_damages_id) REFERENCES economic_damages(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_medical_expenses_economic ON medical_expenses(economic_damages_id);
CREATE INDEX IF NOT EXISTS idx_medical_expenses_date ON medical_expenses(date);

-- ============= NON-ECONOMIC DAMAGES =============

CREATE TABLE IF NOT EXISTS non_economic_damages (
    id TEXT PRIMARY KEY,
    settlement_calculation_id TEXT NOT NULL,

    pain_and_suffering REAL NOT NULL DEFAULT 0.0,
    emotional_distress REAL NOT NULL DEFAULT 0.0,
    loss_of_consortium REAL NOT NULL DEFAULT 0.0,
    loss_of_enjoyment_of_life REAL NOT NULL DEFAULT 0.0,
    disfigurement REAL NOT NULL DEFAULT 0.0,
    loss_of_reputation REAL NOT NULL DEFAULT 0.0,

    total_non_economic REAL NOT NULL DEFAULT 0.0,

    -- Calculation methodology
    methodology TEXT NOT NULL DEFAULT 'Multiplier', -- Multiplier, PerDiem, Comparable, Hybrid
    multiplier REAL NOT NULL DEFAULT 2.5,
    per_diem_rate REAL DEFAULT NULL,
    days_in_pain INTEGER DEFAULT NULL,

    FOREIGN KEY (settlement_calculation_id) REFERENCES settlement_calculations(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_non_economic_calc ON non_economic_damages(settlement_calculation_id);

-- ============= PUNITIVE DAMAGES =============

CREATE TABLE IF NOT EXISTS punitive_damages (
    id TEXT PRIMARY KEY,
    settlement_calculation_id TEXT NOT NULL,

    amount REAL NOT NULL,
    basis TEXT NOT NULL,
    defendant_net_worth REAL DEFAULT NULL,
    reprehensibility_score REAL NOT NULL, -- 0.0-1.0
    likelihood TEXT NOT NULL, -- Unlikely, Possible, Probable, HighlyLikely

    FOREIGN KEY (settlement_calculation_id) REFERENCES settlement_calculations(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_punitive_calc ON punitive_damages(settlement_calculation_id);

-- ============= LIABILITY ANALYSIS =============

CREATE TABLE IF NOT EXISTS liability_analysis (
    id TEXT PRIMARY KEY,
    settlement_calculation_id TEXT NOT NULL,

    plaintiff_liability_percentage REAL NOT NULL DEFAULT 0.0,
    defendant_liability_percentage REAL NOT NULL DEFAULT 100.0,
    comparative_negligence_applies BOOLEAN NOT NULL DEFAULT TRUE,
    jurisdiction TEXT NOT NULL,
    liability_strength TEXT NOT NULL, -- Clear, Strong, Moderate, Weak, Disputed

    FOREIGN KEY (settlement_calculation_id) REFERENCES settlement_calculations(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_liability_calc ON liability_analysis(settlement_calculation_id);

-- ============= LIABILITY FACTORS =============

CREATE TABLE IF NOT EXISTS liability_factors (
    id TEXT PRIMARY KEY,
    liability_analysis_id TEXT NOT NULL,

    factor TEXT NOT NULL,
    favors TEXT NOT NULL, -- Plaintiff or Defendant
    weight REAL NOT NULL DEFAULT 0.5, -- 0.0-1.0

    FOREIGN KEY (liability_analysis_id) REFERENCES liability_analysis(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_liability_factors_analysis ON liability_factors(liability_analysis_id);

-- ============= RISK ASSESSMENT =============

CREATE TABLE IF NOT EXISTS risk_assessment (
    id TEXT PRIMARY KEY,
    settlement_calculation_id TEXT NOT NULL,

    trial_risk_score REAL NOT NULL, -- 0.0 (low risk) to 1.0 (high risk)
    trial_cost_estimate REAL NOT NULL,
    expected_trial_duration_months INTEGER NOT NULL,
    probability_of_win REAL NOT NULL, -- 0.0-1.0
    expected_trial_value REAL NOT NULL,

    FOREIGN KEY (settlement_calculation_id) REFERENCES settlement_calculations(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_risk_calc ON risk_assessment(settlement_calculation_id);

-- ============= CASE STRENGTHS =============

CREATE TABLE IF NOT EXISTS case_strengths (
    id TEXT PRIMARY KEY,
    risk_assessment_id TEXT NOT NULL,

    description TEXT NOT NULL,
    impact TEXT NOT NULL, -- Critical, Major, Moderate, Minor

    FOREIGN KEY (risk_assessment_id) REFERENCES risk_assessment(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_strengths_risk ON case_strengths(risk_assessment_id);

-- ============= CASE WEAKNESSES =============

CREATE TABLE IF NOT EXISTS case_weaknesses (
    id TEXT PRIMARY KEY,
    risk_assessment_id TEXT NOT NULL,

    description TEXT NOT NULL,
    impact TEXT NOT NULL, -- Critical, Major, Moderate, Minor
    mitigation TEXT DEFAULT NULL,

    FOREIGN KEY (risk_assessment_id) REFERENCES risk_assessment(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_weaknesses_risk ON case_weaknesses(risk_assessment_id);

-- ============= COMPARABLE VERDICTS =============

CREATE TABLE IF NOT EXISTS comparable_verdicts (
    id TEXT PRIMARY KEY,
    settlement_calculation_id TEXT NOT NULL,

    case_name TEXT NOT NULL,
    jurisdiction TEXT NOT NULL,
    year INTEGER NOT NULL,
    case_type TEXT NOT NULL,
    injury_type TEXT NOT NULL,
    verdict_amount REAL NOT NULL,
    economic_damages REAL NOT NULL,
    non_economic_damages REAL NOT NULL,
    similarity_score REAL NOT NULL, -- 0.0-1.0
    citation TEXT DEFAULT NULL,

    FOREIGN KEY (settlement_calculation_id) REFERENCES settlement_calculations(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_comparables_calc ON comparable_verdicts(settlement_calculation_id);
CREATE INDEX IF NOT EXISTS idx_comparables_jurisdiction ON comparable_verdicts(jurisdiction);
CREATE INDEX IF NOT EXISTS idx_comparables_year ON comparable_verdicts(year);

-- ============= SETTLEMENT OFFERS =============

CREATE TABLE IF NOT EXISTS settlement_offers (
    id TEXT PRIMARY KEY,
    matter_id TEXT NOT NULL,
    settlement_calculation_id TEXT NOT NULL,

    offer_from TEXT NOT NULL, -- Plaintiff or Defendant
    offer_amount REAL NOT NULL,
    offer_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expiration_date TIMESTAMP DEFAULT NULL,

    status TEXT NOT NULL DEFAULT 'Pending', -- Pending, Accepted, Rejected, Countered, Expired
    response TEXT DEFAULT NULL,
    response_date TIMESTAMP DEFAULT NULL,

    -- Analysis
    percentage_of_demand REAL NOT NULL DEFAULT 0.0,
    percentage_of_calculated_value REAL NOT NULL DEFAULT 0.0,
    comparison_to_verdict_range TEXT DEFAULT NULL,
    net_recovery_after_costs REAL NOT NULL DEFAULT 0.0,

    recommendation TEXT NOT NULL DEFAULT 'NeedsClientInput', -- Accept, Reject, Counter, NeedsClientInput

    FOREIGN KEY (matter_id) REFERENCES cases(id) ON DELETE CASCADE,
    FOREIGN KEY (settlement_calculation_id) REFERENCES settlement_calculations(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_offers_matter ON settlement_offers(matter_id);
CREATE INDEX IF NOT EXISTS idx_offers_calc ON settlement_offers(settlement_calculation_id);
CREATE INDEX IF NOT EXISTS idx_offers_date ON settlement_offers(offer_date);

-- ============= SETTLEMENT OFFER TERMS =============

CREATE TABLE IF NOT EXISTS settlement_terms (
    id TEXT PRIMARY KEY,
    settlement_offer_id TEXT NOT NULL,

    term TEXT NOT NULL,
    value REAL DEFAULT NULL,

    FOREIGN KEY (settlement_offer_id) REFERENCES settlement_offers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_terms_offer ON settlement_terms(settlement_offer_id);

-- ============= SETTLEMENT OFFER CONDITIONS =============

CREATE TABLE IF NOT EXISTS settlement_conditions (
    id TEXT PRIMARY KEY,
    settlement_offer_id TEXT NOT NULL,

    condition TEXT NOT NULL,

    FOREIGN KEY (settlement_offer_id) REFERENCES settlement_offers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_conditions_offer ON settlement_conditions(settlement_offer_id);

-- ============= COUNTER OFFERS =============

CREATE TABLE IF NOT EXISTS counter_offers (
    id TEXT PRIMARY KEY,
    settlement_calculation_id TEXT NOT NULL,
    original_offer_id TEXT DEFAULT NULL,

    amount REAL NOT NULL,
    date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    rationale TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'Pending',

    FOREIGN KEY (settlement_calculation_id) REFERENCES settlement_calculations(id) ON DELETE CASCADE,
    FOREIGN KEY (original_offer_id) REFERENCES settlement_offers(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_counter_calc ON counter_offers(settlement_calculation_id);
CREATE INDEX IF NOT EXISTS idx_counter_date ON counter_offers(date);

-- ============= DEMAND LETTERS =============

CREATE TABLE IF NOT EXISTS demand_letters (
    id TEXT PRIMARY KEY,
    settlement_calculation_id TEXT NOT NULL,
    matter_id TEXT NOT NULL,

    recipient_name TEXT NOT NULL,
    recipient_address TEXT NOT NULL,
    subject TEXT NOT NULL,
    opening_paragraph TEXT NOT NULL,
    facts_section TEXT NOT NULL,
    liability_section TEXT NOT NULL,
    damages_section TEXT NOT NULL,
    settlement_demand REAL NOT NULL,
    deadline TIMESTAMP NOT NULL,
    closing_paragraph TEXT NOT NULL,

    letter_html TEXT NOT NULL,
    letter_pdf_path TEXT DEFAULT NULL,

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by TEXT NOT NULL,
    sent_at TIMESTAMP DEFAULT NULL,

    FOREIGN KEY (settlement_calculation_id) REFERENCES settlement_calculations(id) ON DELETE CASCADE,
    FOREIGN KEY (matter_id) REFERENCES cases(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_demand_calc ON demand_letters(settlement_calculation_id);
CREATE INDEX IF NOT EXISTS idx_demand_matter ON demand_letters(matter_id);
CREATE INDEX IF NOT EXISTS idx_demand_created ON demand_letters(created_at);

-- ============= DEMAND LETTER EXHIBITS =============

CREATE TABLE IF NOT EXISTS demand_exhibits (
    id TEXT PRIMARY KEY,
    demand_letter_id TEXT NOT NULL,

    exhibit_letter TEXT NOT NULL,
    description TEXT NOT NULL,
    file_path TEXT NOT NULL,

    FOREIGN KEY (demand_letter_id) REFERENCES demand_letters(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_exhibits_demand ON demand_exhibits(demand_letter_id);

-- ============= CALCULATION NOTES =============

CREATE TABLE IF NOT EXISTS calculation_notes (
    id TEXT PRIMARY KEY,
    settlement_calculation_id TEXT NOT NULL,

    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    author TEXT NOT NULL,
    note TEXT NOT NULL,
    note_type TEXT NOT NULL DEFAULT 'General', -- Assumption, Adjustment, ClientInput, ExpertOpinion, LegalCitation, General

    FOREIGN KEY (settlement_calculation_id) REFERENCES settlement_calculations(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_notes_calc ON calculation_notes(settlement_calculation_id);
CREATE INDEX IF NOT EXISTS idx_notes_timestamp ON calculation_notes(timestamp);

-- ============= MEDICAL TREATMENT TIMELINE =============

CREATE TABLE IF NOT EXISTS treatment_events (
    id TEXT PRIMARY KEY,
    settlement_calculation_id TEXT NOT NULL,

    date TIMESTAMP NOT NULL,
    event_type TEXT NOT NULL, -- InitialEmergency, Hospitalization, Surgery, etc.
    provider TEXT NOT NULL,
    description TEXT NOT NULL,
    cost REAL NOT NULL,
    was_emergency BOOLEAN NOT NULL DEFAULT FALSE,
    related_to_incident BOOLEAN NOT NULL DEFAULT TRUE,

    FOREIGN KEY (settlement_calculation_id) REFERENCES settlement_calculations(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_treatment_calc ON treatment_events(settlement_calculation_id);
CREATE INDEX IF NOT EXISTS idx_treatment_date ON treatment_events(date);

-- ============= AI SETTLEMENT ANALYSIS =============

CREATE TABLE IF NOT EXISTS ai_settlement_analysis (
    id TEXT PRIMARY KEY,
    settlement_calculation_id TEXT NOT NULL,

    predicted_settlement_value REAL NOT NULL,
    confidence_score REAL NOT NULL, -- 0.0-1.0
    prediction_model_version TEXT NOT NULL,
    similar_cases_analyzed INTEGER NOT NULL,

    -- Judge data
    judge_name TEXT DEFAULT NULL,
    judge_avg_plaintiff_verdict REAL DEFAULT NULL,
    judge_plaintiff_win_rate REAL DEFAULT NULL,
    judge_trials_presided INTEGER DEFAULT NULL,

    -- Opposing counsel data
    opposing_firm TEXT DEFAULT NULL,
    opposing_attorney TEXT DEFAULT NULL,
    counsel_avg_settlement_pct REAL DEFAULT NULL,
    counsel_trial_rate REAL DEFAULT NULL,

    -- Insurance company data
    insurance_company TEXT DEFAULT NULL,
    insurance_avg_time_to_settle INTEGER DEFAULT NULL,
    insurance_settlement_pct REAL DEFAULT NULL,
    insurance_litigation_rate REAL DEFAULT NULL,

    -- Venue statistics
    venue_county TEXT DEFAULT NULL,
    venue_avg_plaintiff_verdict REAL DEFAULT NULL,
    venue_plaintiff_win_rate REAL DEFAULT NULL,
    venue_median_time_to_trial INTEGER DEFAULT NULL,

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (settlement_calculation_id) REFERENCES settlement_calculations(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_ai_calc ON ai_settlement_analysis(settlement_calculation_id);

-- ============= AI FACTORS =============

CREATE TABLE IF NOT EXISTS ai_factors (
    id TEXT PRIMARY KEY,
    ai_analysis_id TEXT NOT NULL,

    factor_name TEXT NOT NULL,
    importance REAL NOT NULL, -- 0.0-1.0
    impact_direction TEXT NOT NULL, -- Positive, Negative, Neutral
    description TEXT NOT NULL,

    FOREIGN KEY (ai_analysis_id) REFERENCES ai_settlement_analysis(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_ai_factors_analysis ON ai_factors(ai_analysis_id);

-- ============= STRUCTURED SETTLEMENTS =============

CREATE TABLE IF NOT EXISTS structured_settlements (
    id TEXT PRIMARY KEY,
    settlement_calculation_id TEXT NOT NULL,

    total_value REAL NOT NULL,
    upfront_payment REAL NOT NULL,
    present_value REAL NOT NULL,
    discount_rate REAL NOT NULL,

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (settlement_calculation_id) REFERENCES settlement_calculations(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_structured_calc ON structured_settlements(settlement_calculation_id);

-- ============= PERIODIC PAYMENTS =============

CREATE TABLE IF NOT EXISTS periodic_payments (
    id TEXT PRIMARY KEY,
    structured_settlement_id TEXT NOT NULL,

    amount REAL NOT NULL,
    frequency TEXT NOT NULL, -- Monthly, Quarterly, Annually, Lump
    duration_years INTEGER NOT NULL,
    start_date TIMESTAMP NOT NULL,

    FOREIGN KEY (structured_settlement_id) REFERENCES structured_settlements(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_periodic_structured ON periodic_payments(structured_settlement_id);
