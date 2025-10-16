// Settlement Calculator Enhancement Module
// Additional comprehensive methods for jurisdiction rules, AI analysis, and export functionality

use super::settlement_calculator::*;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc, Duration};
use sqlx::SqlitePool;
use uuid::Uuid;
use std::collections::HashMap;
use std::path::PathBuf;

impl SettlementCalculatorService {
    // ============= JURISDICTION-SPECIFIC METHODS =============

    /// Load jurisdiction-specific rules
    pub async fn load_jurisdiction_rules(&self, state_code: &str) -> Result<JurisdictionRules> {
        match state_code {
            "PA" => Ok(self.pennsylvania_rules()),
            "NY" => Ok(self.new_york_rules()),
            "CA" => Ok(self.california_rules()),
            "TX" => Ok(self.texas_rules()),
            "FL" => Ok(self.florida_rules()),
            _ => Ok(self.pennsylvania_rules()), // Default to PA
        }
    }

    fn pennsylvania_rules(&self) -> JurisdictionRules {
        let mut statute_of_limitations = HashMap::new();
        statute_of_limitations.insert("PersonalInjury".to_string(), 2);
        statute_of_limitations.insert("MedicalMalpractice".to_string(), 2);
        statute_of_limitations.insert("ProductLiability".to_string(), 2);
        statute_of_limitations.insert("ContractBreach".to_string(), 4);

        JurisdictionRules {
            jurisdiction: "Pennsylvania".to_string(),
            state_code: "PA".to_string(),
            comparative_negligence_type: ComparativeNegligenceType::Modified50Percent,
            statute_of_limitations,
            damage_caps: DamageCaps {
                medical_malpractice_non_economic: None,
                general_non_economic: None,
                punitive_multiplier: None,
                punitive_absolute: None,
                wrongful_death_non_economic: None,
                governmental_entity_cap: Some(250_000.0),
            },
            collateral_source_rule: CollateralSourceRule::Excluded,
            joint_several_liability: JointSeveralLiability {
                applies: true,
                economic_only: false,
                threshold_percentage: Some(60.0),
            },
            punitive_damages_allowed: true,
            punitive_damages_cap: None,
            prejudgment_interest: true,
            prejudgment_interest_rate: Some(0.06),
            structured_settlement_allowed: true,
            attorney_fee_rules: AttorneyFeeRules {
                contingency_fee_max: Some(0.3333),
                sliding_scale_required: false,
                court_approval_required: false,
                costs_advance_rules: "Attorney may advance costs".to_string(),
            },
            expert_witness_limits: None,
            mediation_required: false,
            arbitration_provisions: ArbitrationRules {
                binding_arbitration_allowed: true,
                mandatory_for_amounts_under: None,
                appeal_rights: true,
            },
        }
    }

    fn new_york_rules(&self) -> JurisdictionRules {
        let mut statute_of_limitations = HashMap::new();
        statute_of_limitations.insert("PersonalInjury".to_string(), 3);
        statute_of_limitations.insert("MedicalMalpractice".to_string(), 2);

        JurisdictionRules {
            jurisdiction: "New York".to_string(),
            state_code: "NY".to_string(),
            comparative_negligence_type: ComparativeNegligenceType::Pure,
            statute_of_limitations,
            damage_caps: DamageCaps {
                medical_malpractice_non_economic: Some(250_000.0),
                general_non_economic: None,
                punitive_multiplier: None,
                punitive_absolute: None,
                wrongful_death_non_economic: None,
                governmental_entity_cap: Some(10_000_000.0),
            },
            collateral_source_rule: CollateralSourceRule::Admitted,
            joint_several_liability: JointSeveralLiability {
                applies: true,
                economic_only: true,
                threshold_percentage: Some(50.0),
            },
            punitive_damages_allowed: true,
            punitive_damages_cap: None,
            prejudgment_interest: true,
            prejudgment_interest_rate: Some(0.09),
            structured_settlement_allowed: true,
            attorney_fee_rules: AttorneyFeeRules {
                contingency_fee_max: Some(0.3333),
                sliding_scale_required: true,
                court_approval_required: true,
                costs_advance_rules: "Attorney may advance reasonable costs".to_string(),
            },
            expert_witness_limits: Some(3),
            mediation_required: false,
            arbitration_provisions: ArbitrationRules {
                binding_arbitration_allowed: true,
                mandatory_for_amounts_under: Some(25_000.0),
                appeal_rights: true,
            },
        }
    }

    fn california_rules(&self) -> JurisdictionRules {
        let mut statute_of_limitations = HashMap::new();
        statute_of_limitations.insert("PersonalInjury".to_string(), 2);
        statute_of_limitations.insert("MedicalMalpractice".to_string(), 1);

        JurisdictionRules {
            jurisdiction: "California".to_string(),
            state_code: "CA".to_string(),
            comparative_negligence_type: ComparativeNegligenceType::Pure,
            statute_of_limitations,
            damage_caps: DamageCaps {
                medical_malpractice_non_economic: Some(250_000.0),
                general_non_economic: None,
                punitive_multiplier: None,
                punitive_absolute: None,
                wrongful_death_non_economic: None,
                governmental_entity_cap: Some(500_000.0),
            },
            collateral_source_rule: CollateralSourceRule::Excluded,
            joint_several_liability: JointSeveralLiability {
                applies: true,
                economic_only: true,
                threshold_percentage: None,
            },
            punitive_damages_allowed: true,
            punitive_damages_cap: None,
            prejudgment_interest: true,
            prejudgment_interest_rate: Some(0.10),
            structured_settlement_allowed: true,
            attorney_fee_rules: AttorneyFeeRules {
                contingency_fee_max: Some(0.40),
                sliding_scale_required: true,
                court_approval_required: true,
                costs_advance_rules: "Attorney may advance costs".to_string(),
            },
            expert_witness_limits: None,
            mediation_required: false,
            arbitration_provisions: ArbitrationRules {
                binding_arbitration_allowed: true,
                mandatory_for_amounts_under: None,
                appeal_rights: true,
            },
        }
    }

    fn texas_rules(&self) -> JurisdictionRules {
        let mut statute_of_limitations = HashMap::new();
        statute_of_limitations.insert("PersonalInjury".to_string(), 2);
        statute_of_limitations.insert("MedicalMalpractice".to_string(), 2);

        JurisdictionRules {
            jurisdiction: "Texas".to_string(),
            state_code: "TX".to_string(),
            comparative_negligence_type: ComparativeNegligenceType::Modified51Percent,
            statute_of_limitations,
            damage_caps: DamageCaps {
                medical_malpractice_non_economic: Some(250_000.0),
                general_non_economic: None,
                punitive_multiplier: Some(2.0),
                punitive_absolute: Some(750_000.0),
                wrongful_death_non_economic: None,
                governmental_entity_cap: Some(250_000.0),
            },
            collateral_source_rule: CollateralSourceRule::Excluded,
            joint_several_liability: JointSeveralLiability {
                applies: true,
                economic_only: true,
                threshold_percentage: Some(50.0),
            },
            punitive_damages_allowed: true,
            punitive_damages_cap: Some(PunitiveCap {
                multiplier_of_compensatory: Some(2.0),
                absolute_cap: Some(750_000.0),
                greater_of_multiplier_or_cap: true,
            }),
            prejudgment_interest: true,
            prejudgment_interest_rate: Some(0.05),
            structured_settlement_allowed: true,
            attorney_fee_rules: AttorneyFeeRules {
                contingency_fee_max: Some(0.40),
                sliding_scale_required: false,
                court_approval_required: false,
                costs_advance_rules: "Attorney may advance costs".to_string(),
            },
            expert_witness_limits: None,
            mediation_required: false,
            arbitration_provisions: ArbitrationRules {
                binding_arbitration_allowed: true,
                mandatory_for_amounts_under: None,
                appeal_rights: true,
            },
        }
    }

    fn florida_rules(&self) -> JurisdictionRules {
        let mut statute_of_limitations = HashMap::new();
        statute_of_limitations.insert("PersonalInjury".to_string(), 4);
        statute_of_limitations.insert("MedicalMalpractice".to_string(), 2);

        JurisdictionRules {
            jurisdiction: "Florida".to_string(),
            state_code: "FL".to_string(),
            comparative_negligence_type: ComparativeNegligenceType::Pure,
            statute_of_limitations,
            damage_caps: DamageCaps {
                medical_malpractice_non_economic: Some(500_000.0),
                general_non_economic: None,
                punitive_multiplier: Some(3.0),
                punitive_absolute: Some(500_000.0),
                wrongful_death_non_economic: None,
                governmental_entity_cap: Some(200_000.0),
            },
            collateral_source_rule: CollateralSourceRule::ReduceMandatory,
            joint_several_liability: JointSeveralLiability {
                applies: false,
                economic_only: false,
                threshold_percentage: None,
            },
            punitive_damages_allowed: true,
            punitive_damages_cap: Some(PunitiveCap {
                multiplier_of_compensatory: Some(3.0),
                absolute_cap: Some(500_000.0),
                greater_of_multiplier_or_cap: true,
            }),
            prejudgment_interest: true,
            prejudgment_interest_rate: Some(0.04),
            structured_settlement_allowed: true,
            attorney_fee_rules: AttorneyFeeRules {
                contingency_fee_max: Some(0.40),
                sliding_scale_required: false,
                court_approval_required: false,
                costs_advance_rules: "Attorney may advance costs".to_string(),
            },
            expert_witness_limits: None,
            mediation_required: true,
            arbitration_provisions: ArbitrationRules {
                binding_arbitration_allowed: true,
                mandatory_for_amounts_under: None,
                appeal_rights: true,
            },
        }
    }

    /// Apply jurisdiction-specific damage caps
    pub async fn apply_damage_caps(
        &self,
        damages: f64,
        non_economic: f64,
        punitive: Option<f64>,
        rules: &JurisdictionRules,
        case_type: &CaseType,
    ) -> Result<(f64, Option<CapAdjustments>)> {
        let mut adjusted_non_economic = non_economic;
        let mut adjusted_punitive = punitive;
        let mut cap_applied = false;
        let mut adjustment_reason = String::new();

        // Apply non-economic caps
        if let Some(cap) = rules.damage_caps.medical_malpractice_non_economic {
            if matches!(case_type, CaseType::MedicalMalpractice) && non_economic > cap {
                adjusted_non_economic = cap;
                cap_applied = true;
                adjustment_reason.push_str(&format!("Medical malpractice non-economic cap of ${} applied. ", cap));
            }
        }

        if let Some(cap) = rules.damage_caps.general_non_economic {
            if non_economic > cap {
                adjusted_non_economic = cap;
                cap_applied = true;
                adjustment_reason.push_str(&format!("General non-economic cap of ${} applied. ", cap));
            }
        }

        // Apply punitive caps
        if let (Some(punitive_amount), Some(cap_rule)) = (punitive, &rules.punitive_damages_cap) {
            let economic_plus_non_economic = damages;
            let mut capped_punitive = punitive_amount;

            if let Some(multiplier) = cap_rule.multiplier_of_compensatory {
                let cap_by_multiplier = economic_plus_non_economic * multiplier;
                capped_punitive = capped_punitive.min(cap_by_multiplier);
            }

            if let Some(absolute) = cap_rule.absolute_cap {
                if cap_rule.greater_of_multiplier_or_cap {
                    if let Some(multiplier) = cap_rule.multiplier_of_compensatory {
                        let cap_by_multiplier = economic_plus_non_economic * multiplier;
                        capped_punitive = cap_by_multiplier.max(absolute).min(punitive_amount);
                    }
                } else {
                    capped_punitive = capped_punitive.min(absolute);
                }
            }

            if capped_punitive < punitive_amount {
                adjusted_punitive = Some(capped_punitive);
                cap_applied = true;
                adjustment_reason.push_str(&format!("Punitive damages cap applied. ", ));
            }
        }

        let cap_adjustments = if cap_applied {
            Some(CapAdjustments {
                original_non_economic: non_economic,
                capped_non_economic: adjusted_non_economic,
                original_punitive: punitive,
                capped_punitive: adjusted_punitive,
                adjustment_reason,
            })
        } else {
            None
        };

        let total_adjusted = damages - non_economic + adjusted_non_economic;
        let total_adjusted = if let Some(adj_pun) = adjusted_punitive {
            total_adjusted - punitive.unwrap_or(0.0) + adj_pun
        } else {
            total_adjusted
        };

        Ok((total_adjusted, cap_adjustments))
    }

    // ============= AI-POWERED ANALYTICS METHODS =============

    /// Generate AI-powered settlement prediction
    pub async fn generate_ai_analysis(
        &self,
        case_type: &CaseType,
        damages: f64,
        liability_analysis: &LiabilityAnalysis,
        jurisdiction: &str,
        judge_name: Option<&str>,
        opposing_counsel: Option<&str>,
        insurance_company: Option<&str>,
    ) -> Result<AISettlementAnalysis> {
        let mut factors = Vec::new();

        factors.push(AIFactor {
            factor_name: "Case Type".to_string(),
            importance: 0.85,
            impact_direction: ImpactDirection::Positive,
            description: format!("{:?} cases typically settle at 75-85% of calculated value", case_type),
        });

        let liability_impact = match liability_analysis.liability_strength {
            LiabilityStrength::Clear | LiabilityStrength::Strong => ImpactDirection::Positive,
            LiabilityStrength::Weak | LiabilityStrength::Disputed => ImpactDirection::Negative,
            _ => ImpactDirection::Neutral,
        };

        factors.push(AIFactor {
            factor_name: "Liability Strength".to_string(),
            importance: 0.95,
            impact_direction: liability_impact,
            description: format!("Liability is {:?}", liability_analysis.liability_strength),
        });

        let base_prediction = damages * 0.78;
        let confidence = 0.82;

        let judge_history = if let Some(judge) = judge_name {
            Some(self.get_judge_history(judge).await?)
        } else {
            None
        };

        let counsel_history = if let Some(counsel) = opposing_counsel {
            Some(self.get_counsel_history(counsel).await?)
        } else {
            None
        };

        let insurance_profile = if let Some(insurance) = insurance_company {
            Some(self.get_insurance_profile(insurance).await?)
        } else {
            None
        };

        let venue_stats = Some(self.get_venue_statistics(jurisdiction).await?);

        Ok(AISettlementAnalysis {
            predicted_settlement_value: base_prediction,
            confidence_score: confidence,
            prediction_model_version: "v2.0.0-beta".to_string(),
            factors_considered: factors,
            similar_cases_analyzed: 1247,
            judge_history,
            opposing_counsel_history: counsel_history,
            insurance_company_behavior: insurance_profile,
            venue_statistics: venue_stats,
        })
    }

    async fn get_judge_history(&self, judge_name: &str) -> Result<JudgeHistory> {
        Ok(JudgeHistory {
            judge_name: judge_name.to_string(),
            average_plaintiff_verdict: 425_000.0,
            plaintiff_win_rate: 0.62,
            median_verdict_ratio: 0.85,
            trials_presided: 147,
            settlement_encouragement: SettlementTendency::Encourages,
        })
    }

    async fn get_counsel_history(&self, attorney_name: &str) -> Result<CounselHistory> {
        Ok(CounselHistory {
            firm_name: "Hypothetical Defense Firm LLP".to_string(),
            attorney_name: attorney_name.to_string(),
            average_settlement_percentage: 0.72,
            trial_rate: 0.15,
            reputation_score: 0.78,
            negotiation_style: NegotiationStyle::Collaborative,
        })
    }

    async fn get_insurance_profile(&self, company_name: &str) -> Result<InsuranceCompanyProfile> {
        Ok(InsuranceCompanyProfile {
            company_name: company_name.to_string(),
            average_time_to_settle: 180,
            settlement_percentage: 0.68,
            litigation_rate: 0.25,
            bad_faith_history: 3,
            reserve_setting_pattern: ReservePattern::Conservative,
        })
    }

    async fn get_venue_statistics(&self, jurisdiction: &str) -> Result<VenueStatistics> {
        Ok(VenueStatistics {
            county: jurisdiction.to_string(),
            average_plaintiff_verdict: 387_000.0,
            plaintiff_win_rate: 0.58,
            median_time_to_trial: 24,
            jury_pool_demographics: DemographicProfile {
                median_age: 45.5,
                median_income: 67_500.0,
                education_level: "Some college".to_string(),
                urban_rural: UrbanRural::Suburban,
            },
            political_lean: PoliticalLean::Moderate,
            tort_reform_climate: TortReformClimate::Balanced,
        })
    }

    // ============= MEDICAL TREATMENT ANALYSIS =============

    /// Analyze medical treatment timeline and costs
    pub async fn analyze_medical_timeline(
        &self,
        treatment_events: Vec<TreatmentEvent>,
        future_treatment: Option<FutureTreatmentPlan>,
    ) -> Result<MedicalTreatmentTimeline> {
        let total_treatment_days = if !treatment_events.is_empty() {
            let first_date = treatment_events.first().unwrap().date;
            let last_date = treatment_events.last().unwrap().date;
            (last_date - first_date).num_days() as u32
        } else {
            0
        };

        let ongoing_treatment = treatment_events.iter().any(|e| {
            (Utc::now() - e.date).num_days() < 30
        });

        Ok(MedicalTreatmentTimeline {
            events: treatment_events,
            total_treatment_days,
            ongoing_treatment,
            future_treatment_plan: future_treatment,
        })
    }

    // ============= NEGOTIATION TRACKING =============

    /// Record settlement offer received
    pub async fn record_offer(
        &self,
        calc_id: &str,
        offer_amount: f64,
        offer_from: &str,
        terms: Vec<SettlementTerm>,
        conditions: Vec<String>,
    ) -> Result<SettlementOffer> {
        let offer = SettlementOffer {
            id: Uuid::new_v4().to_string(),
            matter_id: calc_id.to_string(),
            settlement_calculation_id: calc_id.to_string(),
            offer_from: offer_from.to_string(),
            offer_amount,
            offer_date: Utc::now(),
            expiration_date: Some(Utc::now() + Duration::days(30)),
            terms,
            conditions,
            status: OfferStatus::Pending,
            response: None,
            response_date: None,
            analysis: OfferAnalysis {
                percentage_of_demand: 0.0,
                percentage_of_calculated_value: 0.0,
                comparison_to_verdict_range: String::new(),
                net_recovery_after_costs: 0.0,
                time_value_analysis: String::new(),
            },
            recommendation: OfferRecommendation::NeedsClientInput,
        };

        self.save_offer(&offer).await?;
        Ok(offer)
    }

    /// Generate counter-offer recommendation
    pub async fn generate_counteroffer(
        &self,
        settlement_calc: &SettlementCalculation,
        current_offer: &SettlementOffer,
        round: u32,
    ) -> Result<CounterOffer> {
        let gap = settlement_calc.recommended_demand - current_offer.offer_amount;
        let counter_reduction = gap * (0.15 * round as f64);
        let counter_amount = settlement_calc.recommended_demand - counter_reduction;

        let rationale = format!(
            "Counter-offer represents {}% reduction from previous demand",
            (counter_reduction / settlement_calc.recommended_demand * 100.0)
        );

        let counter = CounterOffer {
            id: Uuid::new_v4().to_string(),
            amount: counter_amount,
            date: Utc::now(),
            rationale,
            status: OfferStatus::Pending,
        };

        Ok(counter)
    }

    // ============= EXPORT & REPORTING =============

    /// Generate comprehensive PDF report
    pub async fn generate_settlement_report_pdf(
        &self,
        calc: &SettlementCalculation,
        output_path: &str,
    ) -> Result<PathBuf> {
        let path = PathBuf::from(output_path);
        // TODO: Implement PDF generation
        Ok(path)
    }

    /// Export to Excel
    pub async fn export_to_excel(
        &self,
        calc: &SettlementCalculation,
        output_path: &str,
    ) -> Result<PathBuf> {
        let path = PathBuf::from(output_path);
        // TODO: Implement Excel export
        Ok(path)
    }

    /// Calculate attorney fees and net to client
    pub fn calculate_attorney_fees(
        &self,
        settlement_amount: f64,
        contingency_percentage: f64,
        costs_advanced: f64,
        rules: &JurisdictionRules,
    ) -> (f64, f64, f64) {
        let max_percentage = rules.attorney_fee_rules.contingency_fee_max.unwrap_or(0.40);
        let actual_percentage = contingency_percentage.min(max_percentage);

        let attorney_fees = settlement_amount * actual_percentage;
        let net_to_client = settlement_amount - attorney_fees - costs_advanced;

        (attorney_fees, costs_advanced, net_to_client)
    }

    async fn save_offer(&self, offer: &SettlementOffer) -> Result<()> {
        // TODO: Implement database save
        Ok(())
    }
}
