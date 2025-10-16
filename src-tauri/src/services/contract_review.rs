// Contract Review & Analysis AI Service
// Automated contract analysis, clause extraction, risk assessment, and redlining

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;
use std::collections::HashMap;
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAnalysis {
    pub id: String,
    pub contract_id: String,
    pub contract_name: String,
    pub contract_type: ContractType,

    // Analysis results
    pub risk_score: f64,  // 0.0 (low risk) to 1.0 (high risk)
    pub risk_level: RiskLevel,
    pub summary: String,

    // Extracted information
    pub parties: Vec<ContractParty>,
    pub effective_date: Option<DateTime<Utc>>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub term_length: Option<String>,
    pub jurisdiction: Option<String>,
    pub governing_law: Option<String>,

    // Clause analysis
    pub clauses_found: Vec<ClauseAnalysis>,
    pub clauses_missing: Vec<MissingClause>,
    pub non_standard_clauses: Vec<NonStandardClause>,
    pub obligations: Vec<Obligation>,

    // Financial terms
    pub payment_terms: Vec<PaymentTerm>,
    pub total_contract_value: Option<f64>,

    // Risks and issues
    pub risks: Vec<ContractRisk>,
    pub issues: Vec<ContractIssue>,
    pub recommendations: Vec<String>,

    // Metadata
    pub analyzed_at: DateTime<Utc>,
    pub analyzed_by: String,
    pub analysis_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContractType {
    Employment,
    Service_agreement,
    Non_disclosure,
    Lease,
    Purchase,
    License,
    Partnership,
    Consulting,
    Vendor,
    Settlement,
    Retainer,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractParty {
    pub name: String,
    pub role: PartyRole,
    pub address: Option<String>,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub is_client: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PartyRole {
    Client,
    Vendor,
    Service_provider,
    Employer,
    Employee,
    Landlord,
    Tenant,
    Buyer,
    Seller,
    Licensor,
    Licensee,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClauseAnalysis {
    pub clause_type: StandardClauseType,
    pub text: String,
    pub location: ClauseLocation,
    pub is_standard: bool,
    pub risk_level: RiskLevel,
    pub notes: Vec<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StandardClauseType {
    Termination,
    Confidentiality,
    Indemnification,
    Limitation_of_liability,
    Force_majeure,
    Governing_law,
    Dispute_resolution,
    Arbitration,
    Severability,
    Entire_agreement,
    Amendment,
    Assignment,
    Notice,
    Warranty,
    Intellectual_property,
    Payment_terms,
    Scope_of_work,
    Compliance,
    Insurance,
    Representations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClauseLocation {
    pub section: Option<String>,
    pub page: Option<u32>,
    pub paragraph: Option<u32>,
    pub start_position: Option<usize>,
    pub end_position: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingClause {
    pub clause_type: StandardClauseType,
    pub importance: ClauseImportance,
    pub reason: String,
    pub template_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClauseImportance {
    Critical,     // Must have
    Important,    // Should have
    Recommended,  // Nice to have
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonStandardClause {
    pub text: String,
    pub location: ClauseLocation,
    pub concern_level: RiskLevel,
    pub explanation: String,
    pub alternative: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obligation {
    pub party: String,
    pub description: String,
    pub deadline: Option<DateTime<Utc>>,
    pub is_recurring: bool,
    pub frequency: Option<String>,
    pub penalty_for_breach: Option<String>,
    pub related_clause: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTerm {
    pub amount: Option<f64>,
    pub currency: String,
    pub description: String,
    pub due_date: Option<DateTime<Utc>>,
    pub frequency: Option<String>,
    pub payment_method: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractRisk {
    pub risk_type: RiskType,
    pub severity: RiskLevel,
    pub description: String,
    pub affected_clause: Option<String>,
    pub mitigation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskType {
    Legal,
    Financial,
    Operational,
    Compliance,
    Reputational,
    Ambiguity,
    One_sided_terms,
    Unlimited_liability,
    Automatic_renewal,
    Restrictive_covenant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractIssue {
    pub issue_type: IssueType,
    pub severity: RiskLevel,
    pub description: String,
    pub location: ClauseLocation,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueType {
    Ambiguous_language,
    Conflicting_terms,
    Missing_definition,
    Vague_deadline,
    Unclear_obligation,
    Unbalanced_terms,
    Excessive_penalty,
    Broad_indemnity,
    Missing_signature,
    Incorrect_date,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractComparison {
    pub contract1_id: String,
    pub contract2_id: String,
    pub comparison_type: ComparisonType,

    // Differences
    pub added_clauses: Vec<ClauseDifference>,
    pub removed_clauses: Vec<ClauseDifference>,
    pub modified_clauses: Vec<ClauseDifference>,

    // Summary
    pub similarity_score: f64,  // 0.0 to 1.0
    pub major_changes_count: u32,
    pub minor_changes_count: u32,

    // Metadata
    pub compared_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComparisonType {
    Version_compare,     // Compare versions of same contract
    Template_compare,    // Compare against standard template
    Redline,             // Generate redline document
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClauseDifference {
    pub clause_type: Option<StandardClauseType>,
    pub old_text: Option<String>,
    pub new_text: Option<String>,
    pub change_type: ChangeType,
    pub impact: RiskLevel,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
    Moved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedlineDocument {
    pub id: String,
    pub original_contract_id: String,
    pub revised_contract_id: String,
    pub redline_html: String,
    pub redline_pdf_path: Option<String>,
    pub changes_summary: Vec<ClauseDifference>,
    pub created_at: DateTime<Utc>,
}

pub struct ContractReviewService {
    db: SqlitePool,
}

impl ContractReviewService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    // ============= Contract Analysis =============

    /// Analyze a contract document
    pub async fn analyze_contract(
        &self,
        contract_id: &str,
        contract_text: &str,
        contract_type: ContractType,
        analyzed_by: &str,
    ) -> Result<ContractAnalysis> {
        let analysis_id = Uuid::new_v4().to_string();

        // Extract basic information
        let parties = self.extract_parties(contract_text).await?;
        let dates = self.extract_dates(contract_text).await?;
        let jurisdiction = self.extract_jurisdiction(contract_text).await?;

        // Analyze clauses
        let clauses_found = self.analyze_clauses(contract_text, &contract_type).await?;
        let clauses_missing = self.find_missing_clauses(&clauses_found, &contract_type).await?;
        let non_standard = self.identify_non_standard_clauses(contract_text, &clauses_found).await?;

        // Extract obligations and payment terms
        let obligations = self.extract_obligations(contract_text, &parties).await?;
        let payment_terms = self.extract_payment_terms(contract_text).await?;

        // Identify risks and issues
        let risks = self.identify_risks(contract_text, &clauses_found, &non_standard).await?;
        let issues = self.identify_issues(contract_text, &clauses_found).await?;

        // Calculate risk score
        let risk_score = self.calculate_risk_score(&risks, &issues, &clauses_missing).await?;
        let risk_level = self.determine_risk_level(risk_score);

        // Generate recommendations
        let recommendations = self.generate_recommendations(
            &clauses_missing,
            &risks,
            &issues,
            &non_standard,
        ).await?;

        // Generate summary
        let summary = self.generate_summary(
            &contract_type,
            risk_score,
            &clauses_missing,
            &risks.len(),
        ).await?;

        let total_value = payment_terms.iter()
            .filter_map(|p| p.amount)
            .sum::<f64>();

        let analysis = ContractAnalysis {
            id: analysis_id,
            contract_id: contract_id.to_string(),
            contract_name: format!("Contract {}", contract_id),
            contract_type,
            risk_score,
            risk_level,
            summary,
            parties,
            effective_date: dates.effective_date,
            expiration_date: dates.expiration_date,
            term_length: dates.term_length,
            jurisdiction,
            governing_law: None,
            clauses_found,
            clauses_missing,
            non_standard_clauses: non_standard,
            obligations,
            payment_terms,
            total_contract_value: if total_value > 0.0 { Some(total_value) } else { None },
            risks,
            issues,
            recommendations,
            analyzed_at: Utc::now(),
            analyzed_by: analyzed_by.to_string(),
            analysis_version: "1.0.0".to_string(),
        };

        self.save_analysis(&analysis).await?;

        Ok(analysis)
    }

    // ============= Clause Extraction =============

    async fn analyze_clauses(
        &self,
        text: &str,
        contract_type: &ContractType,
    ) -> Result<Vec<ClauseAnalysis>> {
        let mut clauses = Vec::new();

        // Extract termination clause
        if let Some(termination) = self.extract_termination_clause(text).await? {
            clauses.push(termination);
        }

        // Extract confidentiality clause
        if let Some(confidentiality) = self.extract_confidentiality_clause(text).await? {
            clauses.push(confidentiality);
        }

        // Extract indemnification clause
        if let Some(indemnification) = self.extract_indemnification_clause(text).await? {
            clauses.push(indemnification);
        }

        // Extract limitation of liability clause
        if let Some(limitation) = self.extract_limitation_of_liability_clause(text).await? {
            clauses.push(limitation);
        }

        // Extract governing law clause
        if let Some(governing_law) = self.extract_governing_law_clause(text).await? {
            clauses.push(governing_law);
        }

        // Extract dispute resolution clause
        if let Some(dispute) = self.extract_dispute_resolution_clause(text).await? {
            clauses.push(dispute);
        }

        Ok(clauses)
    }

    async fn extract_termination_clause(&self, text: &str) -> Result<Option<ClauseAnalysis>> {
        // Look for termination section
        let patterns = vec![
            r"(?i)(termination|term and termination)[\s\S]{0,500}",
            r"(?i)either party may terminate[\s\S]{0,300}",
            r"(?i)this agreement.{0,50}may be terminated[\s\S]{0,300}",
        ];

        for pattern in patterns {
            let re = Regex::new(pattern)?;
            if let Some(matched) = re.find(text) {
                let clause_text = matched.as_str().to_string();

                // Analyze termination terms
                let mut notes = Vec::new();
                let mut risk_level = RiskLevel::Low;

                // Check for notice period
                if !clause_text.to_lowercase().contains("notice") &&
                   !clause_text.to_lowercase().contains("days") {
                    notes.push("No clear notice period specified".to_string());
                    risk_level = RiskLevel::Medium;
                }

                // Check for termination for convenience
                if clause_text.to_lowercase().contains("for any reason") ||
                   clause_text.to_lowercase().contains("without cause") {
                    notes.push("Allows termination for convenience".to_string());
                }

                return Ok(Some(ClauseAnalysis {
                    clause_type: StandardClauseType::Termination,
                    text: clause_text,
                    location: ClauseLocation {
                        section: None,
                        page: None,
                        paragraph: None,
                        start_position: Some(matched.start()),
                        end_position: Some(matched.end()),
                    },
                    is_standard: true,
                    risk_level,
                    notes,
                    suggestions: Vec::new(),
                }));
            }
        }

        Ok(None)
    }

    async fn extract_confidentiality_clause(&self, text: &str) -> Result<Option<ClauseAnalysis>> {
        let patterns = vec![
            r"(?i)(confidentiality|confidential information)[\s\S]{0,500}",
            r"(?i)non-disclosure[\s\S]{0,300}",
        ];

        for pattern in patterns {
            let re = Regex::new(pattern)?;
            if let Some(matched) = re.find(text) {
                let clause_text = matched.as_str().to_string();

                let mut notes = Vec::new();
                let mut risk_level = RiskLevel::Low;

                // Check for definition of confidential information
                if !clause_text.to_lowercase().contains("means") &&
                   !clause_text.to_lowercase().contains("defined as") {
                    notes.push("Confidential information not clearly defined".to_string());
                    risk_level = RiskLevel::Medium;
                }

                // Check for exclusions
                if !clause_text.to_lowercase().contains("public domain") {
                    notes.push("Standard exclusions may be missing".to_string());
                }

                return Ok(Some(ClauseAnalysis {
                    clause_type: StandardClauseType::Confidentiality,
                    text: clause_text,
                    location: ClauseLocation {
                        section: None,
                        page: None,
                        paragraph: None,
                        start_position: Some(matched.start()),
                        end_position: Some(matched.end()),
                    },
                    is_standard: true,
                    risk_level,
                    notes,
                    suggestions: Vec::new(),
                }));
            }
        }

        Ok(None)
    }

    async fn extract_indemnification_clause(&self, text: &str) -> Result<Option<ClauseAnalysis>> {
        let patterns = vec![
            r"(?i)(indemnification|indemnify)[\s\S]{0,500}",
            r"(?i)hold harmless[\s\S]{0,300}",
        ];

        for pattern in patterns {
            let re = Regex::new(pattern)?;
            if let Some(matched) = re.find(text) {
                let clause_text = matched.as_str().to_string();

                let mut notes = Vec::new();
                let mut risk_level = RiskLevel::Medium;

                // Check if one-sided
                let indemnify_count = clause_text.to_lowercase().matches("shall indemnify").count();
                if indemnify_count == 1 {
                    notes.push("One-sided indemnification - only one party indemnifies".to_string());
                    risk_level = RiskLevel::High;
                }

                // Check for unlimited liability
                if !clause_text.to_lowercase().contains("except") &&
                   !clause_text.to_lowercase().contains("limitation") {
                    notes.push("No limitations on indemnification - potentially unlimited liability".to_string());
                    risk_level = RiskLevel::High;
                }

                return Ok(Some(ClauseAnalysis {
                    clause_type: StandardClauseType::Indemnification,
                    text: clause_text,
                    location: ClauseLocation {
                        section: None,
                        page: None,
                        paragraph: None,
                        start_position: Some(matched.start()),
                        end_position: Some(matched.end()),
                    },
                    is_standard: false,  // Indemnification clauses often need customization
                    risk_level,
                    notes,
                    suggestions: vec![
                        "Consider mutual indemnification".to_string(),
                        "Add cap on indemnification liability".to_string(),
                    ],
                }));
            }
        }

        Ok(None)
    }

    async fn extract_limitation_of_liability_clause(&self, text: &str) -> Result<Option<ClauseAnalysis>> {
        let patterns = vec![
            r"(?i)(limitation of liability|liability limit)[\s\S]{0,500}",
            r"(?i)in no event shall[\s\S]{0,300}liable",
        ];

        for pattern in patterns {
            let re = Regex::new(pattern)?;
            if let Some(matched) = re.find(text) {
                let clause_text = matched.as_str().to_string();

                let mut notes = Vec::new();
                let risk_level = RiskLevel::Low;

                // Check for cap amount
                if clause_text.to_lowercase().contains("aggregate") ||
                   clause_text.to_lowercase().contains("exceed") {
                    notes.push("Includes liability cap".to_string());
                }

                // Check for excluded damages
                if clause_text.to_lowercase().contains("consequential") ||
                   clause_text.to_lowercase().contains("indirect") {
                    notes.push("Excludes consequential damages".to_string());
                }

                return Ok(Some(ClauseAnalysis {
                    clause_type: StandardClauseType::Limitation_of_liability,
                    text: clause_text,
                    location: ClauseLocation {
                        section: None,
                        page: None,
                        paragraph: None,
                        start_position: Some(matched.start()),
                        end_position: Some(matched.end()),
                    },
                    is_standard: true,
                    risk_level,
                    notes,
                    suggestions: Vec::new(),
                }));
            }
        }

        Ok(None)
    }

    async fn extract_governing_law_clause(&self, text: &str) -> Result<Option<ClauseAnalysis>> {
        let patterns = vec![
            r"(?i)(governing law|choice of law)[\s\S]{0,200}",
            r"(?i)construed in accordance with[\s\S]{0,150}",
        ];

        for pattern in patterns {
            let re = Regex::new(pattern)?;
            if let Some(matched) = re.find(text) {
                return Ok(Some(ClauseAnalysis {
                    clause_type: StandardClauseType::Governing_law,
                    text: matched.as_str().to_string(),
                    location: ClauseLocation {
                        section: None,
                        page: None,
                        paragraph: None,
                        start_position: Some(matched.start()),
                        end_position: Some(matched.end()),
                    },
                    is_standard: true,
                    risk_level: RiskLevel::Low,
                    notes: Vec::new(),
                    suggestions: Vec::new(),
                }));
            }
        }

        Ok(None)
    }

    async fn extract_dispute_resolution_clause(&self, text: &str) -> Result<Option<ClauseAnalysis>> {
        let patterns = vec![
            r"(?i)(dispute resolution|arbitration)[\s\S]{0,500}",
            r"(?i)disputes arising[\s\S]{0,300}",
        ];

        for pattern in patterns {
            let re = Regex::new(pattern)?;
            if let Some(matched) = re.find(text) {
                let clause_text = matched.as_str().to_string();

                let mut notes = Vec::new();

                if clause_text.to_lowercase().contains("arbitration") {
                    notes.push("Requires arbitration for disputes".to_string());
                }

                if clause_text.to_lowercase().contains("mediation") {
                    notes.push("Includes mediation requirement".to_string());
                }

                return Ok(Some(ClauseAnalysis {
                    clause_type: StandardClauseType::Dispute_resolution,
                    text: clause_text,
                    location: ClauseLocation {
                        section: None,
                        page: None,
                        paragraph: None,
                        start_position: Some(matched.start()),
                        end_position: Some(matched.end()),
                    },
                    is_standard: true,
                    risk_level: RiskLevel::Low,
                    notes,
                    suggestions: Vec::new(),
                }));
            }
        }

        Ok(None)
    }

    // ============= Missing Clause Detection =============

    async fn find_missing_clauses(
        &self,
        found_clauses: &[ClauseAnalysis],
        contract_type: &ContractType,
    ) -> Result<Vec<MissingClause>> {
        let found_types: Vec<StandardClauseType> = found_clauses.iter()
            .map(|c| c.clause_type.clone())
            .collect();

        let mut missing = Vec::new();

        // Check for critical clauses
        if !found_types.contains(&StandardClauseType::Termination) {
            missing.push(MissingClause {
                clause_type: StandardClauseType::Termination,
                importance: ClauseImportance::Critical,
                reason: "Termination clause is essential to define how parties can exit the agreement".to_string(),
                template_text: Some("Either party may terminate this Agreement upon thirty (30) days written notice to the other party.".to_string()),
            });
        }

        if !found_types.contains(&StandardClauseType::Governing_law) {
            missing.push(MissingClause {
                clause_type: StandardClauseType::Governing_law,
                importance: ClauseImportance::Critical,
                reason: "Governing law clause specifies which jurisdiction's laws apply".to_string(),
                template_text: Some("This Agreement shall be governed by and construed in accordance with the laws of the Commonwealth of Pennsylvania.".to_string()),
            });
        }

        if !found_types.contains(&StandardClauseType::Entire_agreement) {
            missing.push(MissingClause {
                clause_type: StandardClauseType::Entire_agreement,
                importance: ClauseImportance::Important,
                reason: "Entire agreement clause prevents claims based on prior negotiations".to_string(),
                template_text: Some("This Agreement constitutes the entire agreement between the parties and supersedes all prior agreements and understandings.".to_string()),
            });
        }

        if !found_types.contains(&StandardClauseType::Severability) {
            missing.push(MissingClause {
                clause_type: StandardClauseType::Severability,
                importance: ClauseImportance::Important,
                reason: "Severability clause ensures rest of contract remains valid if one provision is unenforceable".to_string(),
                template_text: Some("If any provision of this Agreement is held to be invalid or unenforceable, the remaining provisions shall continue in full force and effect.".to_string()),
            });
        }

        // Contract-specific missing clauses
        match contract_type {
            ContractType::Service_agreement | ContractType::Consulting => {
                if !found_types.contains(&StandardClauseType::Scope_of_work) {
                    missing.push(MissingClause {
                        clause_type: StandardClauseType::Scope_of_work,
                        importance: ClauseImportance::Critical,
                        reason: "Service agreements should clearly define the scope of work".to_string(),
                        template_text: None,
                    });
                }

                if !found_types.contains(&StandardClauseType::Payment_terms) {
                    missing.push(MissingClause {
                        clause_type: StandardClauseType::Payment_terms,
                        importance: ClauseImportance::Critical,
                        reason: "Payment terms must be clearly specified".to_string(),
                        template_text: None,
                    });
                }
            }
            ContractType::Non_disclosure => {
                if !found_types.contains(&StandardClauseType::Confidentiality) {
                    missing.push(MissingClause {
                        clause_type: StandardClauseType::Confidentiality,
                        importance: ClauseImportance::Critical,
                        reason: "NDA must include confidentiality obligations".to_string(),
                        template_text: None,
                    });
                }
            }
            _ => {}
        }

        Ok(missing)
    }

    // ============= Risk Assessment =============

    async fn identify_non_standard_clauses(
        &self,
        text: &str,
        found_clauses: &[ClauseAnalysis],
    ) -> Result<Vec<NonStandardClause>> {
        let mut non_standard = Vec::new();

        // Check for automatic renewal clause
        let auto_renewal_re = Regex::new(r"(?i)(automatic.{0,20}renew|automatically renew)")?;
        if let Some(matched) = auto_renewal_re.find(text) {
            non_standard.push(NonStandardClause {
                text: matched.as_str().to_string(),
                location: ClauseLocation {
                    section: None,
                    page: None,
                    paragraph: None,
                    start_position: Some(matched.start()),
                    end_position: Some(matched.end()),
                },
                concern_level: RiskLevel::Medium,
                explanation: "Automatic renewal can trap parties in unwanted contract extensions".to_string(),
                alternative: Some("Require active renewal notice instead of automatic renewal".to_string()),
            });
        }

        // Check for non-compete clause
        let non_compete_re = Regex::new(r"(?i)(non-compete|non compete|shall not compete)")?;
        if let Some(matched) = non_compete_re.find(text) {
            non_standard.push(NonStandardClause {
                text: matched.as_str().to_string(),
                location: ClauseLocation {
                    section: None,
                    page: None,
                    paragraph: None,
                    start_position: Some(matched.start()),
                    end_position: Some(matched.end()),
                },
                concern_level: RiskLevel::High,
                explanation: "Non-compete clauses may be unenforceable if overly broad in scope or duration".to_string(),
                alternative: Some("Limit geographic scope and time period to reasonable terms".to_string()),
            });
        }

        Ok(non_standard)
    }

    async fn identify_risks(
        &self,
        text: &str,
        clauses: &[ClauseAnalysis],
        non_standard: &[NonStandardClause],
    ) -> Result<Vec<ContractRisk>> {
        let mut risks = Vec::new();

        // Check for unlimited liability
        let unlimited_liability_re = Regex::new(r"(?i)(unlimited|without limit)")?;
        if unlimited_liability_re.is_match(text) {
            risks.push(ContractRisk {
                risk_type: RiskType::Unlimited_liability,
                severity: RiskLevel::Critical,
                description: "Contract may expose party to unlimited liability".to_string(),
                affected_clause: Some("Indemnification".to_string()),
                mitigation: "Add liability cap or limit to direct damages only".to_string(),
            });
        }

        // Check for one-sided indemnification
        for clause in clauses {
            if clause.clause_type == StandardClauseType::Indemnification {
                if clause.risk_level == RiskLevel::High {
                    risks.push(ContractRisk {
                        risk_type: RiskType::One_sided_terms,
                        severity: RiskLevel::High,
                        description: "Indemnification clause is one-sided".to_string(),
                        affected_clause: Some("Indemnification".to_string()),
                        mitigation: "Negotiate mutual indemnification".to_string(),
                    });
                }
            }
        }

        // Add risks from non-standard clauses
        for ns_clause in non_standard {
            if ns_clause.concern_level == RiskLevel::High || ns_clause.concern_level == RiskLevel::Critical {
                risks.push(ContractRisk {
                    risk_type: RiskType::Restrictive_covenant,
                    severity: ns_clause.concern_level.clone(),
                    description: ns_clause.explanation.clone(),
                    affected_clause: None,
                    mitigation: ns_clause.alternative.clone().unwrap_or_default(),
                });
            }
        }

        Ok(risks)
    }

    async fn identify_issues(
        &self,
        text: &str,
        clauses: &[ClauseAnalysis],
    ) -> Result<Vec<ContractIssue>> {
        let mut issues = Vec::new();

        // Check for vague deadlines
        let vague_deadline_re = Regex::new(r"(?i)(reasonable time|promptly|as soon as possible)")?;
        if let Some(matched) = vague_deadline_re.find(text) {
            issues.push(ContractIssue {
                issue_type: IssueType::Vague_deadline,
                severity: RiskLevel::Medium,
                description: "Contract contains vague time references".to_string(),
                location: ClauseLocation {
                    section: None,
                    page: None,
                    paragraph: None,
                    start_position: Some(matched.start()),
                    end_position: Some(matched.end()),
                },
                recommended_action: "Replace with specific number of days".to_string(),
            });
        }

        // Check for ambiguous language
        let ambiguous_re = Regex::new(r"(?i)(may or may not|if necessary|as needed)")?;
        if let Some(matched) = ambiguous_re.find(text) {
            issues.push(ContractIssue {
                issue_type: IssueType::Ambiguous_language,
                severity: RiskLevel::Medium,
                description: "Contract contains ambiguous language".to_string(),
                location: ClauseLocation {
                    section: None,
                    page: None,
                    paragraph: None,
                    start_position: Some(matched.start()),
                    end_position: Some(matched.end()),
                },
                recommended_action: "Use clear, definitive language".to_string(),
            });
        }

        Ok(issues)
    }

    async fn calculate_risk_score(
        &self,
        risks: &[ContractRisk],
        issues: &[ContractIssue],
        missing_clauses: &[MissingClause],
    ) -> Result<f64> {
        let mut score = 0.0;

        // Risk contribution (max 0.5)
        for risk in risks {
            match risk.severity {
                RiskLevel::Critical => score += 0.15,
                RiskLevel::High => score += 0.10,
                RiskLevel::Medium => score += 0.05,
                RiskLevel::Low => score += 0.02,
            }
        }

        // Issue contribution (max 0.3)
        for issue in issues {
            match issue.severity {
                RiskLevel::Critical => score += 0.10,
                RiskLevel::High => score += 0.06,
                RiskLevel::Medium => score += 0.03,
                RiskLevel::Low => score += 0.01,
            }
        }

        // Missing clause contribution (max 0.2)
        for missing in missing_clauses {
            match missing.importance {
                ClauseImportance::Critical => score += 0.08,
                ClauseImportance::Important => score += 0.04,
                ClauseImportance::Recommended => score += 0.02,
            }
        }

        // Cap at 1.0
        Ok(score.min(1.0))
    }

    fn determine_risk_level(&self, score: f64) -> RiskLevel {
        if score >= 0.75 {
            RiskLevel::Critical
        } else if score >= 0.5 {
            RiskLevel::High
        } else if score >= 0.25 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    // ============= Information Extraction =============

    async fn extract_parties(&self, text: &str) -> Result<Vec<ContractParty>> {
        let mut parties = Vec::new();

        // Simple party extraction - look for "between X and Y"
        let between_re = Regex::new(r"(?i)between\s+(.+?)\s+and\s+(.+?)[\.,]")?;
        if let Some(caps) = between_re.captures(text) {
            if let (Some(party1), Some(party2)) = (caps.get(1), caps.get(2)) {
                parties.push(ContractParty {
                    name: party1.as_str().trim().to_string(),
                    role: PartyRole::Other,
                    address: None,
                    contact_person: None,
                    email: None,
                    is_client: false,
                });

                parties.push(ContractParty {
                    name: party2.as_str().trim().to_string(),
                    role: PartyRole::Other,
                    address: None,
                    contact_person: None,
                    email: None,
                    is_client: false,
                });
            }
        }

        Ok(parties)
    }

    struct ExtractedDates {
        effective_date: Option<DateTime<Utc>>,
        expiration_date: Option<DateTime<Utc>>,
        term_length: Option<String>,
    }

    async fn extract_dates(&self, text: &str) -> Result<ExtractedDates> {
        // Stub - would use date parsing library
        Ok(ExtractedDates {
            effective_date: None,
            expiration_date: None,
            term_length: None,
        })
    }

    async fn extract_jurisdiction(&self, text: &str) -> Result<Option<String>> {
        // Look for state/jurisdiction mentions
        let jurisdiction_re = Regex::new(r"(?i)(state of|commonwealth of)\s+([A-Za-z\s]+)")?;
        if let Some(caps) = jurisdiction_re.captures(text) {
            if let Some(jurisdiction) = caps.get(2) {
                return Ok(Some(jurisdiction.as_str().trim().to_string()));
            }
        }

        Ok(None)
    }

    async fn extract_obligations(&self, text: &str, parties: &[ContractParty]) -> Result<Vec<Obligation>> {
        let mut obligations = Vec::new();

        // Look for "shall" obligations
        let shall_re = Regex::new(r"(?i)([A-Za-z\s]+)\s+shall\s+([^\.]+)\.")?;
        for caps in shall_re.captures_iter(text) {
            if let (Some(party), Some(action)) = (caps.get(1), caps.get(2)) {
                obligations.push(Obligation {
                    party: party.as_str().trim().to_string(),
                    description: action.as_str().trim().to_string(),
                    deadline: None,
                    is_recurring: false,
                    frequency: None,
                    penalty_for_breach: None,
                    related_clause: None,
                });
            }
        }

        Ok(obligations)
    }

    async fn extract_payment_terms(&self, text: &str) -> Result<Vec<PaymentTerm>> {
        let mut payment_terms = Vec::new();

        // Look for dollar amounts
        let amount_re = Regex::new(r"\$([0-9,]+(?:\.[0-9]{2})?)")?;
        for caps in amount_re.captures_iter(text) {
            if let Some(amount_str) = caps.get(1) {
                let amount_cleaned = amount_str.as_str().replace(",", "");
                if let Ok(amount) = amount_cleaned.parse::<f64>() {
                    payment_terms.push(PaymentTerm {
                        amount: Some(amount),
                        currency: "USD".to_string(),
                        description: "Payment".to_string(),
                        due_date: None,
                        frequency: None,
                        payment_method: None,
                    });
                }
            }
        }

        Ok(payment_terms)
    }

    // ============= Recommendations =============

    async fn generate_recommendations(
        &self,
        missing_clauses: &[MissingClause],
        risks: &[ContractRisk],
        issues: &[ContractIssue],
        non_standard: &[NonStandardClause],
    ) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        // Recommendations for missing clauses
        for missing in missing_clauses {
            if missing.importance == ClauseImportance::Critical {
                recommendations.push(format!(
                    "Add {} clause: {}",
                    format!("{:?}", missing.clause_type).replace("_", " "),
                    missing.reason
                ));
            }
        }

        // Recommendations for risks
        for risk in risks {
            if risk.severity == RiskLevel::Critical || risk.severity == RiskLevel::High {
                recommendations.push(format!("Risk mitigation: {}", risk.mitigation));
            }
        }

        // Recommendations for issues
        for issue in issues {
            if issue.severity == RiskLevel::High {
                recommendations.push(issue.recommended_action.clone());
            }
        }

        Ok(recommendations)
    }

    async fn generate_summary(
        &self,
        contract_type: &ContractType,
        risk_score: f64,
        missing_clauses: &[MissingClause],
        risk_count: &usize,
    ) -> Result<String> {
        let risk_desc = if risk_score >= 0.75 {
            "critical risk level"
        } else if risk_score >= 0.5 {
            "high risk level"
        } else if risk_score >= 0.25 {
            "medium risk level"
        } else {
            "low risk level"
        };

        let critical_missing = missing_clauses.iter()
            .filter(|m| m.importance == ClauseImportance::Critical)
            .count();

        Ok(format!(
            "This {} has a {} (score: {:.2}). Found {} risks and {} critical missing clauses. Review recommended before signing.",
            format!("{:?}", contract_type).replace("_", " "),
            risk_desc,
            risk_score,
            risk_count,
            critical_missing
        ))
    }

    // ============= Contract Comparison =============

    /// Compare two contract versions
    pub async fn compare_contracts(
        &self,
        contract1_id: &str,
        contract2_id: &str,
        comparison_type: ComparisonType,
    ) -> Result<ContractComparison> {
        // Stub - would implement diff algorithm
        Ok(ContractComparison {
            contract1_id: contract1_id.to_string(),
            contract2_id: contract2_id.to_string(),
            comparison_type,
            added_clauses: Vec::new(),
            removed_clauses: Vec::new(),
            modified_clauses: Vec::new(),
            similarity_score: 0.85,
            major_changes_count: 0,
            minor_changes_count: 0,
            compared_at: Utc::now(),
        })
    }

    /// Generate redline document
    pub async fn generate_redline(
        &self,
        original_id: &str,
        revised_id: &str,
    ) -> Result<RedlineDocument> {
        let comparison = self.compare_contracts(
            original_id,
            revised_id,
            ComparisonType::Redline,
        ).await?;

        // Stub - would generate HTML/PDF with track changes formatting
        Ok(RedlineDocument {
            id: Uuid::new_v4().to_string(),
            original_contract_id: original_id.to_string(),
            revised_contract_id: revised_id.to_string(),
            redline_html: "<html>Redline document</html>".to_string(),
            redline_pdf_path: None,
            changes_summary: comparison.modified_clauses,
            created_at: Utc::now(),
        })
    }

    // ============= Helper Methods =============

    async fn save_analysis(&self, analysis: &ContractAnalysis) -> Result<()> {
        // Stub - would save to database
        Ok(())
    }
}
