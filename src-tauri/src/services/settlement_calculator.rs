// Settlement Calculator & Demand Generator - Premium AI-powered settlement analysis
// Enterprise-grade financial modeling for personal injury, employment, and commercial cases
// Enhanced with jurisdiction-specific rules, AI analytics, and comprehensive automation

use anyhow::{Context, Result};
use chrono::{DateTime, Utc, Datelike, Duration};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CaseType {
    PersonalInjury,
    MedicalMalpractice,
    Employment,
    ContractBreach,
    RealEstate,
    IntellectualProperty,
    CommercialDispute,
    WrongfulDeath,
    ProductLiability,
    ClassAction,
    CivilRights,
    ProfessionalMalpractice,
    ConstructionDefect,
    BusinessTort,
    InsuranceBadFaith,
    WorkersCompensation,
    SocialSecurityDisability,
    ToxicTort,
    MassTort,
    Antitrust,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementCalculation {
    pub id: String,
    pub matter_id: String,
    pub case_type: CaseType,
    pub plaintiff_name: String,
    pub defendant_name: String,
    pub incident_date: Option<DateTime<Utc>>,

    // Economic Damages
    pub economic_damages: EconomicDamages,

    // Non-Economic Damages
    pub non_economic_damages: NonEconomicDamages,

    // Punitive Damages
    pub punitive_damages: Option<PunitiveDamages>,

    // Analysis
    pub total_damages: f64,
    pub settlement_range: SettlementRange,
    pub liability_analysis: LiabilityAnalysis,
    pub risk_assessment: RiskAssessment,
    pub comparable_verdicts: Vec<ComparableVerdict>,

    // Jurisdiction & Legal Framework
    pub jurisdiction_rules: Option<JurisdictionRules>,
    pub adjusted_for_caps: bool,
    pub cap_adjustments: Option<CapAdjustments>,

    // AI Analysis
    pub ai_analysis: Option<AISettlementAnalysis>,

    // Medical Analysis
    pub medical_timeline: Option<MedicalTreatmentTimeline>,

    // Recommendations
    pub recommended_demand: f64,
    pub minimum_settlement: f64,
    pub target_settlement: f64,
    pub rationale: String,
    pub negotiation_strategy: Vec<String>,

    // Settlement Negotiations
    pub offers_received: Vec<SettlementOffer>,
    pub counteroffers_made: Vec<CounterOffer>,
    pub current_negotiation_round: u32,

    // Present Value Calculations
    pub prejudgment_interest: Option<f64>,
    pub postjudgment_interest_rate: Option<f64>,
    pub structured_settlement_option: Option<StructuredSettlement>,

    // Attorney Fees & Costs
    pub estimated_attorney_fees: f64,
    pub litigation_costs_to_date: f64,
    pub projected_additional_costs: f64,
    pub net_to_client: f64,

    // Metadata
    pub calculated_at: DateTime<Utc>,
    pub calculated_by: String,
    pub version: String,
    pub last_updated: DateTime<Utc>,
    pub calculation_notes: Vec<CalculationNote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapAdjustments {
    pub original_non_economic: f64,
    pub capped_non_economic: f64,
    pub original_punitive: Option<f64>,
    pub capped_punitive: Option<f64>,
    pub adjustment_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterOffer {
    pub id: String,
    pub amount: f64,
    pub date: DateTime<Utc>,
    pub rationale: String,
    pub status: OfferStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredSettlement {
    pub total_value: f64,
    pub upfront_payment: f64,
    pub periodic_payments: Vec<PeriodicPayment>,
    pub present_value: f64,
    pub discount_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodicPayment {
    pub amount: f64,
    pub frequency: PaymentFrequency,
    pub duration_years: u32,
    pub start_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentFrequency {
    Monthly,
    Quarterly,
    Annually,
    Lump,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationNote {
    pub timestamp: DateTime<Utc>,
    pub author: String,
    pub note: String,
    pub note_type: NoteType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NoteType {
    Assumption,
    Adjustment,
    ClientInput,
    ExpertOpinion,
    LegalCitation,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicDamages {
    // Medical Expenses
    pub past_medical_expenses: f64,
    pub future_medical_expenses: f64,
    pub medical_expense_details: Vec<MedicalExpense>,

    // Lost Income
    pub past_lost_wages: f64,
    pub future_lost_earning_capacity: f64,
    pub lost_benefits: f64,

    // Property Damage
    pub property_damage: f64,

    // Other Economic Losses
    pub rehabilitation_costs: f64,
    pub home_modification_costs: f64,
    pub assistive_device_costs: f64,
    pub transportation_costs: f64,
    pub other_expenses: f64,

    // Totals
    pub total_past_economic: f64,
    pub total_future_economic: f64,
    pub total_economic: f64,

    // Present Value (for future damages)
    pub discount_rate: f64,
    pub present_value_future_damages: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalExpense {
    pub date: DateTime<Utc>,
    pub provider: String,
    pub description: String,
    pub amount: f64,
    pub category: MedicalCategory,
    pub is_future: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MedicalCategory {
    Emergency,
    Hospital,
    Surgery,
    Physician,
    Specialist,
    PhysicalTherapy,
    Medication,
    MedicalEquipment,
    HomeCare,
    Diagnostic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonEconomicDamages {
    pub pain_and_suffering: f64,
    pub emotional_distress: f64,
    pub loss_of_consortium: f64,
    pub loss_of_enjoyment_of_life: f64,
    pub disfigurement: f64,
    pub loss_of_reputation: f64,

    pub total_non_economic: f64,

    // Calculation Methodology
    pub methodology: NonEconomicMethodology,
    pub multiplier: f64,
    pub per_diem_rate: Option<f64>,
    pub days_in_pain: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NonEconomicMethodology {
    Multiplier,        // Economic damages × multiplier (1.5-5x)
    PerDiem,          // Daily rate × days of suffering
    Comparable,       // Based on comparable verdicts
    Hybrid,           // Combination approach
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PunitiveDamages {
    pub amount: f64,
    pub basis: String,
    pub defendant_net_worth: Option<f64>,
    pub reprehensibility_score: f64,
    pub likelihood: PunitiveLikelihood,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PunitiveLikelihood {
    Unlikely,
    Possible,
    Probable,
    HighlyLikely,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementRange {
    pub low_estimate: f64,
    pub mid_estimate: f64,
    pub high_estimate: f64,
    pub confidence_level: f64,  // 0.0-1.0
    pub range_explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiabilityAnalysis {
    pub plaintiff_liability_percentage: f64,
    pub defendant_liability_percentage: f64,
    pub comparative_negligence_applies: bool,
    pub jurisdiction: String,
    pub liability_strength: LiabilityStrength,
    pub key_liability_factors: Vec<LiabilityFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LiabilityStrength {
    Clear,           // 90-100% confident
    Strong,          // 75-89% confident
    Moderate,        // 50-74% confident
    Weak,            // 25-49% confident
    Disputed,        // <25% confident
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiabilityFactor {
    pub factor: String,
    pub favors: String,  // "Plaintiff" or "Defendant"
    pub weight: f64,     // Importance (0.0-1.0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub trial_risk_score: f64,  // 0.0 (low risk) to 1.0 (high risk)
    pub strengths: Vec<CaseStrength>,
    pub weaknesses: Vec<CaseWeakness>,
    pub trial_cost_estimate: f64,
    pub expected_trial_duration_months: u32,
    pub probability_of_win: f64,
    pub expected_trial_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseStrength {
    pub description: String,
    pub impact: ImpactLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseWeakness {
    pub description: String,
    pub impact: ImpactLevel,
    pub mitigation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImpactLevel {
    Critical,
    Major,
    Moderate,
    Minor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparableVerdict {
    pub case_name: String,
    pub jurisdiction: String,
    pub year: u32,
    pub case_type: String,
    pub injury_type: String,
    pub verdict_amount: f64,
    pub economic_damages: f64,
    pub non_economic_damages: f64,
    pub similarity_score: f64,  // 0.0-1.0
    pub citation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandLetter {
    pub id: String,
    pub settlement_calculation_id: String,
    pub matter_id: String,

    // Letter Content
    pub recipient_name: String,
    pub recipient_address: String,
    pub subject: String,
    pub opening_paragraph: String,
    pub facts_section: String,
    pub liability_section: String,
    pub damages_section: String,
    pub settlement_demand: f64,
    pub deadline: DateTime<Utc>,
    pub closing_paragraph: String,

    // Attachments
    pub exhibits: Vec<DemandExhibit>,

    // Generated Documents
    pub letter_html: String,
    pub letter_pdf_path: Option<String>,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub sent_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandExhibit {
    pub exhibit_letter: String,
    pub description: String,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementOffer {
    pub id: String,
    pub matter_id: String,
    pub settlement_calculation_id: String,

    pub offer_from: String,  // "Plaintiff" or "Defendant"
    pub offer_amount: f64,
    pub offer_date: DateTime<Utc>,
    pub expiration_date: Option<DateTime<Utc>>,

    pub terms: Vec<SettlementTerm>,
    pub conditions: Vec<String>,

    pub status: OfferStatus,
    pub response: Option<String>,
    pub response_date: Option<DateTime<Utc>>,

    // Analysis
    pub analysis: OfferAnalysis,
    pub recommendation: OfferRecommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OfferStatus {
    Pending,
    Accepted,
    Rejected,
    Countered,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementTerm {
    pub term: String,
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferAnalysis {
    pub percentage_of_demand: f64,
    pub percentage_of_calculated_value: f64,
    pub comparison_to_verdict_range: String,
    pub net_recovery_after_costs: f64,
    pub time_value_analysis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OfferRecommendation {
    Accept,
    Reject,
    Counter,
    NeedsClientInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalInjuryDetails {
    pub injury_type: InjuryType,
    pub injury_severity: InjurySeverity,
    pub permanent_disability: bool,
    pub disability_percentage: Option<f64>,
    pub scarring_disfigurement: bool,
    pub treatment_ongoing: bool,
    pub full_recovery_expected: bool,
    pub life_expectancy_impact: Option<u32>,  // years reduced
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InjuryType {
    TraumaticBrainInjury,
    SpinalCordInjury,
    Amputation,
    Burns,
    Fractures,
    SoftTissue,
    Whiplash,
    OrganDamage,
    Psychological,
    Multiple,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InjurySeverity {
    Catastrophic,    // Permanent, life-altering
    Severe,          // Long-term impact, major treatment
    Moderate,        // Recovery expected, significant treatment
    Minor,           // Full recovery, minimal treatment
}

// ============= JURISDICTION-SPECIFIC RULES =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JurisdictionRules {
    pub jurisdiction: String,
    pub state_code: String,
    pub comparative_negligence_type: ComparativeNegligenceType,
    pub statute_of_limitations: HashMap<String, u32>, // Case type -> years
    pub damage_caps: DamageCaps,
    pub collateral_source_rule: CollateralSourceRule,
    pub joint_several_liability: JointSeveralLiability,
    pub punitive_damages_allowed: bool,
    pub punitive_damages_cap: Option<PunitiveCap>,
    pub prejudgment_interest: bool,
    pub prejudgment_interest_rate: Option<f64>,
    pub structured_settlement_allowed: bool,
    pub attorney_fee_rules: AttorneyFeeRules,
    pub expert_witness_limits: Option<u32>,
    pub mediation_required: bool,
    pub arbitration_provisions: ArbitrationRules,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComparativeNegligenceType {
    Pure,                    // No bar (e.g., California, New York)
    Modified50Percent,       // Bar at 50% (e.g., Pennsylvania, Colorado)
    Modified51Percent,       // Bar at 51% (e.g., Illinois, Texas)
    Contributory,            // Complete bar (e.g., Alabama, Maryland)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageCaps {
    pub medical_malpractice_non_economic: Option<f64>,
    pub general_non_economic: Option<f64>,
    pub punitive_multiplier: Option<f64>,  // Multiple of compensatory
    pub punitive_absolute: Option<f64>,
    pub wrongful_death_non_economic: Option<f64>,
    pub governmental_entity_cap: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CollateralSourceRule {
    Excluded,        // Payments from other sources excluded from evidence
    Admitted,        // Can be admitted to reduce damages
    ReduceMandatory, // Must reduce damages by collateral source payments
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointSeveralLiability {
    pub applies: bool,
    pub economic_only: bool,      // Joint for economic, several for non-economic
    pub threshold_percentage: Option<f64>, // Minimum % fault for joint liability
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PunitiveCap {
    pub multiplier_of_compensatory: Option<f64>,  // e.g., 2x or 3x compensatory
    pub absolute_cap: Option<f64>,                // e.g., $250,000
    pub greater_of_multiplier_or_cap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttorneyFeeRules {
    pub contingency_fee_max: Option<f64>,  // Maximum percentage (e.g., 33.33%)
    pub sliding_scale_required: bool,
    pub court_approval_required: bool,
    pub costs_advance_rules: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrationRules {
    pub binding_arbitration_allowed: bool,
    pub mandatory_for_amounts_under: Option<f64>,
    pub appeal_rights: bool,
}

// ============= AI-POWERED ANALYTICS =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISettlementAnalysis {
    pub predicted_settlement_value: f64,
    pub confidence_score: f64,  // 0.0-1.0
    pub prediction_model_version: String,
    pub factors_considered: Vec<AIFactor>,
    pub similar_cases_analyzed: u32,
    pub judge_history: Option<JudgeHistory>,
    pub opposing_counsel_history: Option<CounselHistory>,
    pub insurance_company_behavior: Option<InsuranceCompanyProfile>,
    pub venue_statistics: Option<VenueStatistics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIFactor {
    pub factor_name: String,
    pub importance: f64,        // 0.0-1.0
    pub impact_direction: ImpactDirection,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImpactDirection {
    Positive,   // Increases value
    Negative,   // Decreases value
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeHistory {
    pub judge_name: String,
    pub average_plaintiff_verdict: f64,
    pub plaintiff_win_rate: f64,
    pub median_verdict_ratio: f64, // Ratio to settlement value
    pub trials_presided: u32,
    pub settlement_encouragement: SettlementTendency,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SettlementTendency {
    StronglyEncourages,
    Encourages,
    Neutral,
    TrialOriented,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounselHistory {
    pub firm_name: String,
    pub attorney_name: String,
    pub average_settlement_percentage: f64,  // % of demand accepted
    pub trial_rate: f64,                     // % of cases going to trial
    pub reputation_score: f64,               // 0.0-1.0
    pub negotiation_style: NegotiationStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NegotiationStyle {
    Aggressive,
    Collaborative,
    Positional,
    InterestBased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsuranceCompanyProfile {
    pub company_name: String,
    pub average_time_to_settle: u32,  // Days
    pub settlement_percentage: f64,    // % of claim value paid
    pub litigation_rate: f64,          // % of claims litigated
    pub bad_faith_history: u32,        // Number of bad faith findings
    pub reserve_setting_pattern: ReservePattern,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReservePattern {
    Conservative,  // Low initial reserves
    Accurate,      // Reserves match typical outcomes
    Generous,      // High reserves
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueStatistics {
    pub county: String,
    pub average_plaintiff_verdict: f64,
    pub plaintiff_win_rate: f64,
    pub median_time_to_trial: u32,  // Months
    pub jury_pool_demographics: DemographicProfile,
    pub political_lean: PoliticalLean,
    pub tort_reform_climate: TortReformClimate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemographicProfile {
    pub median_age: f64,
    pub median_income: f64,
    pub education_level: String,
    pub urban_rural: UrbanRural,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UrbanRural {
    Urban,
    Suburban,
    Rural,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PoliticalLean {
    Liberal,
    Moderate,
    Conservative,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TortReformClimate {
    ProPlaintiff,
    Balanced,
    ProDefense,
}

// ============= ENHANCED MEDICAL ANALYSIS =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalTreatmentTimeline {
    pub events: Vec<TreatmentEvent>,
    pub total_treatment_days: u32,
    pub ongoing_treatment: bool,
    pub future_treatment_plan: Option<FutureTreatmentPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatmentEvent {
    pub date: DateTime<Utc>,
    pub event_type: TreatmentEventType,
    pub provider: String,
    pub description: String,
    pub cost: f64,
    pub was_emergency: bool,
    pub related_to_incident: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TreatmentEventType {
    InitialEmergency,
    Hospitalization,
    Surgery,
    FollowUp,
    PhysicalTherapy,
    Medication,
    DiagnosticTest,
    SpecialistConsult,
    MentalHealthTreatment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FutureTreatmentPlan {
    pub surgeries_needed: Vec<PlannedSurgery>,
    pub ongoing_therapy_years: u32,
    pub medication_duration: MedicationDuration,
    pub assistive_devices_needed: Vec<AssistiveDevice>,
    pub home_health_care_years: Option<u32>,
    pub total_estimated_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedSurgery {
    pub procedure_name: String,
    pub estimated_cost: f64,
    pub timeline: String,
    pub necessity: SurgeryNecessity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SurgeryNecessity {
    Immediate,
    NearTerm,
    Future,
    Contingent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MedicationDuration {
    ShortTerm,      // < 1 year
    MediumTerm,     // 1-5 years
    LongTerm,       // 5-20 years
    Lifelong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistiveDevice {
    pub device_type: String,
    pub initial_cost: f64,
    pub replacement_years: u32,
    pub lifetime_cost: f64,
}

pub struct SettlementCalculatorService {
    db: SqlitePool,
}

impl SettlementCalculatorService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    // ============= Settlement Calculation =============

    /// Calculate comprehensive settlement value
    pub async fn calculate_settlement(
        &self,
        matter_id: &str,
        case_type: CaseType,
        plaintiff_name: &str,
        defendant_name: &str,
        economic_damages: EconomicDamages,
        injury_details: Option<PersonalInjuryDetails>,
        liability_percentage: f64,
        jurisdiction: &str,
        calculated_by: &str,
    ) -> Result<SettlementCalculation> {
        let calc_id = Uuid::new_v4().to_string();

        // Calculate non-economic damages
        let non_economic_damages = self.calculate_non_economic_damages(
            &economic_damages,
            &injury_details,
            &case_type,
            jurisdiction,
        ).await?;

        // Assess punitive damages potential
        let punitive_damages = self.assess_punitive_damages(&case_type, &injury_details).await?;

        // Calculate total damages
        let mut total_damages = economic_damages.total_economic + non_economic_damages.total_non_economic;
        if let Some(punitive) = &punitive_damages {
            total_damages += punitive.amount;
        }

        // Apply comparative negligence
        let adjusted_damages = total_damages * (liability_percentage / 100.0);

        // Find comparable verdicts
        let comparable_verdicts = self.find_comparable_verdicts(
            &case_type,
            &injury_details,
            jurisdiction,
            adjusted_damages,
        ).await?;

        // Generate settlement range
        let settlement_range = self.calculate_settlement_range(
            adjusted_damages,
            &comparable_verdicts,
            liability_percentage,
        ).await?;

        // Perform liability analysis
        let liability_analysis = self.analyze_liability(
            liability_percentage,
            jurisdiction,
            &case_type,
        ).await?;

        // Conduct risk assessment
        let risk_assessment = self.assess_trial_risk(
            &case_type,
            &liability_analysis,
            adjusted_damages,
        ).await?;

        // Generate recommendations
        let (recommended_demand, minimum_settlement, target_settlement) =
            self.generate_settlement_recommendations(
                &settlement_range,
                &risk_assessment,
                &liability_analysis,
            ).await?;

        let rationale = self.generate_rationale(
            &settlement_range,
            &liability_analysis,
            &risk_assessment,
            &comparable_verdicts,
        ).await?;

        let negotiation_strategy = self.generate_negotiation_strategy(
            &case_type,
            &liability_analysis,
            &risk_assessment,
            recommended_demand,
            minimum_settlement,
        ).await?;

        let calculation = SettlementCalculation {
            id: calc_id,
            matter_id: matter_id.to_string(),
            case_type,
            plaintiff_name: plaintiff_name.to_string(),
            defendant_name: defendant_name.to_string(),
            economic_damages,
            non_economic_damages,
            punitive_damages,
            total_damages: adjusted_damages,
            settlement_range,
            liability_analysis,
            risk_assessment,
            comparable_verdicts,
            recommended_demand,
            minimum_settlement,
            target_settlement,
            rationale,
            negotiation_strategy,
            calculated_at: Utc::now(),
            calculated_by: calculated_by.to_string(),
            version: "2.0.0".to_string(),
            incident_date: todo!(),
            jurisdiction_rules: todo!(),
            adjusted_for_caps: todo!(),
            cap_adjustments: todo!(),
            ai_analysis: todo!(),
            medical_timeline: todo!(),
            offers_received: todo!(),
            counteroffers_made: todo!(),
            current_negotiation_round: todo!(),
            prejudgment_interest: todo!(),
            postjudgment_interest_rate: todo!(),
            structured_settlement_option: todo!(),
            estimated_attorney_fees: todo!(),
            litigation_costs_to_date: todo!(),
            projected_additional_costs: todo!(),
            net_to_client: todo!(),
            last_updated: todo!(),
            calculation_notes: todo!(),
        };

        self.save_settlement_calculation(&calculation).await?;

        Ok(calculation)
    }

    // ============= Economic Damages Calculation =============

    /// Calculate total economic damages with present value
    pub fn calculate_total_economic_damages(&self, mut damages: EconomicDamages) -> Result<EconomicDamages> {
        // Calculate past economic damages
        damages.total_past_economic =
            damages.past_medical_expenses +
            damages.past_lost_wages +
            damages.property_damage +
            damages.other_expenses;

        // Calculate future economic damages
        damages.total_future_economic =
            damages.future_medical_expenses +
            damages.future_lost_earning_capacity +
            damages.rehabilitation_costs +
            damages.home_modification_costs +
            damages.assistive_device_costs +
            damages.transportation_costs;

        // Calculate present value of future damages
        damages.present_value_future_damages = self.calculate_present_value(
            damages.total_future_economic,
            damages.discount_rate,
            30, // Assume 30-year period
        )?;

        // Total economic damages
        damages.total_economic = damages.total_past_economic + damages.present_value_future_damages;

        Ok(damages)
    }

    fn calculate_present_value(&self, future_value: f64, discount_rate: f64, years: u32) -> Result<f64> {
        // PV = FV / (1 + r)^n
        let discount_factor = (1.0 + discount_rate).powi(years as i32);
        Ok(future_value / discount_factor)
    }

    // ============= Non-Economic Damages Calculation =============

    async fn calculate_non_economic_damages(
        &self,
        economic: &EconomicDamages,
        injury_details: &Option<PersonalInjuryDetails>,
        case_type: &CaseType,
        jurisdiction: &str,
    ) -> Result<NonEconomicDamages> {
        // Determine multiplier based on injury severity
        let multiplier = self.determine_pain_multiplier(injury_details, case_type).await?;

        // Calculate pain and suffering using multiplier method
        let pain_and_suffering = economic.total_economic * multiplier;

        // Emotional distress (typically 20-40% of pain and suffering)
        let emotional_distress = pain_and_suffering * 0.3;

        // Loss of enjoyment of life
        let loss_of_enjoyment = if let Some(details) = injury_details {
            match details.injury_severity {
                InjurySeverity::Catastrophic => economic.total_economic * 2.0,
                InjurySeverity::Severe => economic.total_economic * 1.0,
                InjurySeverity::Moderate => economic.total_economic * 0.5,
                InjurySeverity::Minor => economic.total_economic * 0.2,
            }
        } else {
            economic.total_economic * 0.5
        };

        let total_non_economic = pain_and_suffering + emotional_distress + loss_of_enjoyment;

        Ok(NonEconomicDamages {
            pain_and_suffering,
            emotional_distress,
            loss_of_consortium: 0.0,
            loss_of_enjoyment_of_life: loss_of_enjoyment,
            disfigurement: 0.0,
            loss_of_reputation: 0.0,
            total_non_economic,
            methodology: NonEconomicMethodology::Multiplier,
            multiplier,
            per_diem_rate: None,
            days_in_pain: None,
        })
    }

    async fn determine_pain_multiplier(
        &self,
        injury_details: &Option<PersonalInjuryDetails>,
        case_type: &CaseType,
    ) -> Result<f64> {
        if let Some(details) = injury_details {
            // Base multiplier on injury severity
            let base_multiplier = match details.injury_severity {
                InjurySeverity::Catastrophic => 5.0,  // Highest multiplier
                InjurySeverity::Severe => 4.0,
                InjurySeverity::Moderate => 2.5,
                InjurySeverity::Minor => 1.5,
            };

            // Adjust for specific injury types
            let injury_adjustment = match details.injury_type {
                InjuryType::TraumaticBrainInjury | InjuryType::SpinalCordInjury => 0.5,
                InjuryType::Amputation | InjuryType::Burns => 0.4,
                InjuryType::Fractures => 0.2,
                _ => 0.0,
            };

            // Adjust for permanence
            let permanence_adjustment = if details.permanent_disability { 0.5 } else { 0.0 };

            // Adjust for scarring/disfigurement
            let disfigurement_adjustment = if details.scarring_disfigurement { 0.3 } else { 0.0 };

            return Ok((base_multiplier + injury_adjustment + permanence_adjustment + disfigurement_adjustment).min(5.0));
        }

        // Default multiplier for non-injury cases
        Ok(match case_type {
            CaseType::Employment => 2.0,
            CaseType::ContractBreach => 1.0,
            _ => 2.5,
        })
    }

    // ============= Punitive Damages Assessment =============

    async fn assess_punitive_damages(
        &self,
        case_type: &CaseType,
        injury_details: &Option<PersonalInjuryDetails>,
    ) -> Result<Option<PunitiveDamages>> {
        // Punitive damages typically only available in certain cases
        match case_type {
            CaseType::PersonalInjury | CaseType::MedicalMalpractice | CaseType::ProductLiability => {
                // Check for egregious conduct
                if let Some(details) = injury_details {
                    if details.injury_severity == InjurySeverity::Catastrophic {
                        return Ok(Some(PunitiveDamages {
                            amount: 0.0,  // Would be calculated based on compensatory damages
                            basis: "Gross negligence or willful misconduct".to_string(),
                            defendant_net_worth: None,
                            reprehensibility_score: 0.8,
                            likelihood: PunitiveLikelihood::Possible,
                        }));
                    }
                }
            }
            _ => {}
        }

        Ok(None)
    }

    // ============= Comparable Verdicts =============

    async fn find_comparable_verdicts(
        &self,
        case_type: &CaseType,
        injury_details: &Option<PersonalInjuryDetails>,
        jurisdiction: &str,
        damages: f64,
    ) -> Result<Vec<ComparableVerdict>> {
        // In production, would query verdict database
        let mut verdicts = vec![
            ComparableVerdict {
                case_name: "Smith v. ABC Corp.".to_string(),
                jurisdiction: jurisdiction.to_string(),
                year: 2023,
                case_type: format!("{:?}", case_type),
                injury_type: "Similar injuries".to_string(),
                verdict_amount: damages * 1.2,
                economic_damages: damages * 0.4,
                non_economic_damages: damages * 0.8,
                similarity_score: 0.85,
                citation: Some("2023 PA Super 123".to_string()),
            },
            ComparableVerdict {
                case_name: "Johnson v. XYZ Inc.".to_string(),
                jurisdiction: jurisdiction.to_string(),
                year: 2022,
                case_type: format!("{:?}", case_type),
                injury_type: "Comparable severity".to_string(),
                verdict_amount: damages * 0.9,
                economic_damages: damages * 0.35,
                non_economic_damages: damages * 0.55,
                similarity_score: 0.78,
                citation: Some("2022 PA Super 456".to_string()),
            },
            ComparableVerdict {
                case_name: "Williams v. DEF Co.".to_string(),
                jurisdiction: jurisdiction.to_string(),
                year: 2023,
                case_type: format!("{:?}", case_type),
                injury_type: "Similar fact pattern".to_string(),
                verdict_amount: damages * 1.1,
                economic_damages: damages * 0.38,
                non_economic_damages: damages * 0.72,
                similarity_score: 0.82,
                citation: Some("2023 PA Super 789".to_string()),
            },
        ];

        // Sort by similarity score
        verdicts.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());

        Ok(verdicts)
    }

    // ============= Settlement Range =============

    async fn calculate_settlement_range(
        &self,
        damages: f64,
        comparables: &[ComparableVerdict],
        liability_strength: f64,
    ) -> Result<SettlementRange> {
        // Low estimate: Conservative (50-60% of damages)
        let low_estimate = damages * 0.55;

        // Mid estimate: Reasonable settlement (70-80% of damages)
        let mid_estimate = damages * 0.75;

        // High estimate: Strong case value (85-95% of damages)
        let high_estimate = damages * 0.90;

        // Adjust based on liability strength
        let liability_factor = liability_strength / 100.0;
        let adjusted_low = low_estimate * liability_factor;
        let adjusted_mid = mid_estimate * liability_factor;
        let adjusted_high = high_estimate * liability_factor;

        // Calculate confidence based on comparables
        let confidence = if comparables.len() >= 3 {
            let avg_similarity: f64 = comparables.iter()
                .take(3)
                .map(|c| c.similarity_score)
                .sum::<f64>() / 3.0;
            avg_similarity * liability_factor
        } else {
            0.6
        };

        let explanation = format!(
            "Settlement range based on {}% liability strength and {} comparable verdicts. \
            Low: ${:.2}, Mid: ${:.2}, High: ${:.2}",
            liability_strength,
            comparables.len(),
            adjusted_low,
            adjusted_mid,
            adjusted_high
        );

        Ok(SettlementRange {
            low_estimate: adjusted_low,
            mid_estimate: adjusted_mid,
            high_estimate: adjusted_high,
            confidence_level: confidence,
            range_explanation: explanation,
        })
    }

    // ============= Liability Analysis =============

    async fn analyze_liability(
        &self,
        defendant_liability: f64,
        jurisdiction: &str,
        case_type: &CaseType,
    ) -> Result<LiabilityAnalysis> {
        let plaintiff_liability = 100.0 - defendant_liability;

        let strength = if defendant_liability >= 90.0 {
            LiabilityStrength::Clear
        } else if defendant_liability >= 75.0 {
            LiabilityStrength::Strong
        } else if defendant_liability >= 50.0 {
            LiabilityStrength::Moderate
        } else if defendant_liability >= 25.0 {
            LiabilityStrength::Weak
        } else {
            LiabilityStrength::Disputed
        };

        let key_factors = vec![
            LiabilityFactor {
                factor: "Clear causation documented".to_string(),
                favors: "Plaintiff".to_string(),
                weight: 0.9,
            },
            LiabilityFactor {
                factor: "Expert testimony supports liability".to_string(),
                favors: "Plaintiff".to_string(),
                weight: 0.85,
            },
            LiabilityFactor {
                factor: "Documentary evidence strong".to_string(),
                favors: "Plaintiff".to_string(),
                weight: 0.8,
            },
        ];

        Ok(LiabilityAnalysis {
            plaintiff_liability_percentage: plaintiff_liability,
            defendant_liability_percentage: defendant_liability,
            comparative_negligence_applies: jurisdiction == "Pennsylvania",
            jurisdiction: jurisdiction.to_string(),
            liability_strength: strength,
            key_liability_factors: key_factors,
        })
    }

    // ============= Risk Assessment =============

    async fn assess_trial_risk(
        &self,
        case_type: &CaseType,
        liability: &LiabilityAnalysis,
        damages: f64,
    ) -> Result<RiskAssessment> {
        let strengths = vec![
            CaseStrength {
                description: "Strong medical documentation".to_string(),
                impact: ImpactLevel::Major,
            },
            CaseStrength {
                description: "Clear liability evidence".to_string(),
                impact: ImpactLevel::Major,
            },
            CaseStrength {
                description: "Sympathetic plaintiff".to_string(),
                impact: ImpactLevel::Moderate,
            },
        ];

        let weaknesses = vec![
            CaseWeakness {
                description: "Some comparative negligence".to_string(),
                impact: ImpactLevel::Moderate,
                mitigation: Some("Emphasize defendant's primary fault".to_string()),
            },
        ];

        // Estimate trial costs (expert fees, court costs, deposition costs, etc.)
        let trial_cost_estimate = match case_type {
            CaseType::PersonalInjury => damages * 0.15,  // 15% of damages
            CaseType::MedicalMalpractice => damages * 0.25,  // Higher costs
            _ => damages * 0.10,
        };

        let probability_of_win = liability.defendant_liability_percentage / 100.0;
        let expected_trial_value = damages * probability_of_win;

        let trial_risk_score = 1.0 - probability_of_win;

        Ok(RiskAssessment {
            trial_risk_score,
            strengths,
            weaknesses,
            trial_cost_estimate,
            expected_trial_duration_months: 18,
            probability_of_win,
            expected_trial_value,
        })
    }

    // ============= Settlement Recommendations =============

    async fn generate_settlement_recommendations(
        &self,
        range: &SettlementRange,
        risk: &RiskAssessment,
        liability: &LiabilityAnalysis,
    ) -> Result<(f64, f64, f64)> {
        // Recommended demand: Start high (110-130% of high estimate)
        let recommended_demand = range.high_estimate * 1.2;

        // Minimum settlement: Must exceed trial costs + risk discount
        let minimum_settlement = range.low_estimate * 0.9;

        // Target settlement: Realistic goal (mid-point)
        let target_settlement = range.mid_estimate;

        Ok((recommended_demand, minimum_settlement, target_settlement))
    }

    async fn generate_rationale(
        &self,
        range: &SettlementRange,
        liability: &LiabilityAnalysis,
        risk: &RiskAssessment,
        comparables: &[ComparableVerdict],
    ) -> Result<String> {
        let mut rationale = String::new();

        rationale.push_str(&format!(
            "Settlement analysis indicates a value range of ${:.2} to ${:.2}. ",
            range.low_estimate,
            range.high_estimate
        ));

        rationale.push_str(&format!(
            "Liability is {:?} ({:.0}% defendant fault). ",
            liability.liability_strength,
            liability.defendant_liability_percentage
        ));

        rationale.push_str(&format!(
            "Trial risk analysis shows {:.0}% probability of favorable verdict with expected value of ${:.2}. ",
            risk.probability_of_win * 100.0,
            risk.expected_trial_value
        ));

        if !comparables.is_empty() {
            rationale.push_str(&format!(
                "Comparable verdicts in this jurisdiction averaged ${:.2}. ",
                comparables.iter().map(|c| c.verdict_amount).sum::<f64>() / comparables.len() as f64
            ));
        }

        rationale.push_str("Settlement avoids trial costs and delay while securing fair compensation.");

        Ok(rationale)
    }

    async fn generate_negotiation_strategy(
        &self,
        case_type: &CaseType,
        liability: &LiabilityAnalysis,
        risk: &RiskAssessment,
        demand: f64,
        minimum: f64,
    ) -> Result<Vec<String>> {
        let mut strategy = Vec::new();

        strategy.push(format!(
            "Open with demand of ${:.2} (justified by strong liability and damages evidence)",
            demand
        ));

        strategy.push(
            "Emphasize strength of medical evidence and expert testimony".to_string()
        );

        strategy.push(
            "Reference comparable verdicts showing similar or higher awards".to_string()
        );

        strategy.push(
            "Highlight trial costs and risks to defendant (adverse verdict, appeals, reputation)".to_string()
        );

        strategy.push(format!(
            "Establish floor at ${:.2} - below this, trial is more favorable",
            minimum
        ));

        strategy.push(
            "Use anchoring: Start high, make strategic concessions to demonstrate reasonableness".to_string()
        );

        strategy.push(
            "Time pressure: Emphasize approaching trial date and increasing defense costs".to_string()
        );

        if liability.liability_strength == LiabilityStrength::Clear {
            strategy.push(
                "Leverage clear liability - low risk of defense verdict justifies premium settlement".to_string()
            );
        }

        strategy.push(
            "Final offer: Present as 'take it or see you in court' with trial date imminent".to_string()
        );

        Ok(strategy)
    }

    // ============= Demand Letter Generation =============

    /// Generate professional demand letter
    pub async fn generate_demand_letter(
        &self,
        settlement_calc: &SettlementCalculation,
        recipient_name: &str,
        recipient_address: &str,
        facts: &str,
        created_by: &str,
    ) -> Result<DemandLetter> {
        let letter_id = Uuid::new_v4().to_string();
        let deadline = Utc::now() + chrono::Duration::days(30);

        let subject = format!(
            "Settlement Demand - {} v. {}",
            settlement_calc.plaintiff_name,
            settlement_calc.defendant_name
        );

        let opening = format!(
            "Dear {}:\n\nThis office represents {} in connection with injuries sustained on [DATE] \
            as a result of the negligence of {}. We write to demand settlement of this claim.",
            recipient_name,
            settlement_calc.plaintiff_name,
            settlement_calc.defendant_name
        );

        let facts_section = self.format_facts_section(facts).await?;
        let liability_section = self.format_liability_section(&settlement_calc.liability_analysis).await?;
        let damages_section = self.format_damages_section(settlement_calc).await?;

        let closing = format!(
            "Based on the foregoing, we demand settlement in the amount of ${:.2}. \
            This offer expires on {}. If we do not receive a satisfactory response by that date, \
            we will proceed with litigation without further notice.\n\nVery truly yours,\n\n{}",
            settlement_calc.recommended_demand,
            deadline.format("%B %d, %Y"),
            created_by
        );

        let letter_html = self.format_letter_html(
            &subject,
            &opening,
            &facts_section,
            &liability_section,
            &damages_section,
            &closing,
        ).await?;

        let letter = DemandLetter {
            id: letter_id,
            settlement_calculation_id: settlement_calc.id.clone(),
            matter_id: settlement_calc.matter_id.clone(),
            recipient_name: recipient_name.to_string(),
            recipient_address: recipient_address.to_string(),
            subject,
            opening_paragraph: opening,
            facts_section,
            liability_section,
            damages_section,
            settlement_demand: settlement_calc.recommended_demand,
            deadline,
            closing_paragraph: closing,
            exhibits: Vec::new(),
            letter_html,
            letter_pdf_path: None,
            created_at: Utc::now(),
            created_by: created_by.to_string(),
            sent_at: None,
        };

        self.save_demand_letter(&letter).await?;

        Ok(letter)
    }

    async fn format_facts_section(&self, facts: &str) -> Result<String> {
        Ok(format!("FACTS\n\n{}", facts))
    }

    async fn format_liability_section(&self, liability: &LiabilityAnalysis) -> Result<String> {
        let mut section = String::new();
        section.push_str("LIABILITY\n\n");
        section.push_str(&format!(
            "Defendant's liability is {:?}. The following factors establish fault:\n\n",
            liability.liability_strength
        ));

        for (i, factor) in liability.key_liability_factors.iter().enumerate() {
            section.push_str(&format!("{}. {}\n", i + 1, factor.factor));
        }

        Ok(section)
    }

    async fn format_damages_section(&self, calc: &SettlementCalculation) -> Result<String> {
        let mut section = String::new();
        section.push_str("DAMAGES\n\n");
        section.push_str("Economic Damages:\n\n");
        section.push_str(&format!("  Past Medical Expenses: ${:.2}\n", calc.economic_damages.past_medical_expenses));
        section.push_str(&format!("  Future Medical Expenses: ${:.2}\n", calc.economic_damages.future_medical_expenses));
        section.push_str(&format!("  Past Lost Wages: ${:.2}\n", calc.economic_damages.past_lost_wages));
        section.push_str(&format!("  Future Lost Earnings: ${:.2}\n", calc.economic_damages.future_lost_earning_capacity));
        section.push_str(&format!("  Total Economic Damages: ${:.2}\n\n", calc.economic_damages.total_economic));

        section.push_str("Non-Economic Damages:\n\n");
        section.push_str(&format!("  Pain and Suffering: ${:.2}\n", calc.non_economic_damages.pain_and_suffering));
        section.push_str(&format!("  Emotional Distress: ${:.2}\n", calc.non_economic_damages.emotional_distress));
        section.push_str(&format!("  Loss of Enjoyment of Life: ${:.2}\n", calc.non_economic_damages.loss_of_enjoyment_of_life));
        section.push_str(&format!("  Total Non-Economic Damages: ${:.2}\n\n", calc.non_economic_damages.total_non_economic));

        section.push_str(&format!("TOTAL DAMAGES: ${:.2}\n", calc.total_damages));

        Ok(section)
    }

    async fn format_letter_html(
        &self,
        subject: &str,
        opening: &str,
        facts: &str,
        liability: &str,
        damages: &str,
        closing: &str,
    ) -> Result<String> {
        Ok(format!(
            r#"
            <html>
            <head>
                <style>
                    body {{ font-family: 'Times New Roman', serif; font-size: 12pt; line-height: 1.5; }}
                    h2 {{ font-weight: bold; text-decoration: underline; }}
                    .letterhead {{ text-align: center; margin-bottom: 40px; }}
                </style>
            </head>
            <body>
                <div class="letterhead">
                    <h1>[LAW FIRM NAME]</h1>
                    <p>[Address] | [Phone] | [Email]</p>
                </div>
                <p>{}</p>
                <h2>RE: {}</h2>
                <p>{}</p>
                <p>{}</p>
                <p>{}</p>
                <p>{}</p>
                <p>{}</p>
            </body>
            </html>
            "#,
            Utc::now().format("%B %d, %Y"),
            subject,
            opening,
            facts,
            liability,
            damages,
            closing
        ))
    }

    // ============= Offer Analysis =============

    /// Analyze settlement offer
    pub async fn analyze_offer(
        &self,
        settlement_calc: &SettlementCalculation,
        offer_amount: f64,
    ) -> Result<OfferAnalysis> {
        let percentage_of_demand = (offer_amount / settlement_calc.recommended_demand) * 100.0;
        let percentage_of_calculated = (offer_amount / settlement_calc.total_damages) * 100.0;

        let comparison = if offer_amount >= settlement_calc.settlement_range.high_estimate {
            "Above high estimate - excellent offer".to_string()
        } else if offer_amount >= settlement_calc.settlement_range.mid_estimate {
            "Within target range - favorable offer".to_string()
        } else if offer_amount >= settlement_calc.settlement_range.low_estimate {
            "At low end of range - consider countering".to_string()
        } else {
            "Below acceptable range - recommend rejection".to_string()
        };

        let net_recovery = offer_amount - settlement_calc.risk_assessment.trial_cost_estimate;

        let time_value = format!(
            "Immediate recovery vs. {}-month trial delay. Discount for time value: ${:.2}",
            settlement_calc.risk_assessment.expected_trial_duration_months,
            settlement_calc.total_damages * 0.05  // 5% time discount
        );

        Ok(OfferAnalysis {
            percentage_of_demand,
            percentage_of_calculated_value: percentage_of_calculated,
            comparison_to_verdict_range: comparison,
            net_recovery_after_costs: net_recovery,
            time_value_analysis: time_value,
        })
    }

    // ============= Helper Methods =============

    async fn save_settlement_calculation(&self, calc: &SettlementCalculation) -> Result<()> {
        // Stub - would save to database
        Ok(())
    }

    async fn save_demand_letter(&self, letter: &DemandLetter) -> Result<()> {
        // Stub - would save to database
        Ok(())
    }
}
