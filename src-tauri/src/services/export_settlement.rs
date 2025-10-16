// Settlement Export Service
// Handles PDF, Excel, and Word export for settlement calculations

use crate::services::settlement_calculator::*;
use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct SettlementExportService;

impl SettlementExportService {
    pub fn new() -> Self {
        Self
    }

    // ============= PDF GENERATION =============

    /// Generate comprehensive PDF report
    pub async fn generate_pdf_report(
        &self,
        calculation: &SettlementCalculation,
        output_path: &str,
    ) -> Result<PathBuf> {
        // In production, this would use a PDF library like printpdf or genpdf
        // For now, generate HTML that can be converted to PDF

        let html = self.generate_report_html(calculation)?;

        // TODO: Use wkhtmltopdf, headless Chrome, or similar to convert HTML to PDF
        // For now, save HTML
        let html_path = PathBuf::from(output_path.replace(".pdf", ".html"));
        std::fs::write(&html_path, html)?;

        Ok(html_path)
    }

    fn generate_report_html(&self, calc: &SettlementCalculation) -> Result<String> {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Settlement Analysis Report</title>
    <style>
        @page {{
            size: letter;
            margin: 1in;
        }}
        body {{
            font-family: 'Georgia', serif;
            font-size: 11pt;
            line-height: 1.6;
            color: #333;
        }}
        .header {{
            text-align: center;
            border-bottom: 3px solid #1e3a5f;
            padding-bottom: 20px;
            margin-bottom: 30px;
        }}
        .header h1 {{
            color: #1e3a5f;
            font-size: 24pt;
            margin: 0 0 10px 0;
        }}
        .header p {{
            color: #666;
            margin: 5px 0;
        }}
        .section {{
            margin-bottom: 30px;
            page-break-inside: avoid;
        }}
        .section-title {{
            color: #1e3a5f;
            font-size: 16pt;
            font-weight: bold;
            border-bottom: 2px solid #d4af37;
            padding-bottom: 5px;
            margin-bottom: 15px;
        }}
        .metric-grid {{
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 20px;
            margin: 20px 0;
        }}
        .metric-box {{
            background: #f8f9fa;
            border: 2px solid #e9ecef;
            border-left: 4px solid #1e3a5f;
            padding: 15px;
        }}
        .metric-label {{
            color: #666;
            font-size: 9pt;
            text-transform: uppercase;
            margin-bottom: 5px;
        }}
        .metric-value {{
            color: #1e3a5f;
            font-size: 20pt;
            font-weight: bold;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 15px 0;
        }}
        th {{
            background: #1e3a5f;
            color: white;
            padding: 12px;
            text-align: left;
            font-weight: bold;
        }}
        td {{
            padding: 10px 12px;
            border-bottom: 1px solid #e9ecef;
        }}
        tr:nth-child(even) {{
            background: #f8f9fa;
        }}
        .highlight-box {{
            background: #fff8e1;
            border-left: 4px solid #d4af37;
            padding: 20px;
            margin: 20px 0;
        }}
        .strategy-list {{
            list-style: none;
            padding: 0;
        }}
        .strategy-list li {{
            background: #f8f9fa;
            margin: 10px 0;
            padding: 15px;
            border-left: 3px solid #4a90e2;
            counter-increment: strategy;
        }}
        .strategy-list li:before {{
            content: counter(strategy) ". ";
            font-weight: bold;
            color: #4a90e2;
            margin-right: 10px;
        }}
        .footer {{
            position: fixed;
            bottom: 0;
            left: 0;
            right: 0;
            text-align: center;
            font-size: 9pt;
            color: #999;
            padding: 10px;
            border-top: 1px solid #e9ecef;
        }}
    </style>
</head>
<body>
    <!-- Header -->
    <div class="header">
        <h1>Settlement Analysis Report</h1>
        <p><strong>{} v. {}</strong></p>
        <p>Matter ID: {} | Jurisdiction: {}</p>
        <p>Calculated: {} | By: {}</p>
    </div>

    <!-- Executive Summary -->
    <div class="section">
        <h2 class="section-title">Executive Summary</h2>
        <div class="metric-grid">
            <div class="metric-box">
                <div class="metric-label">Total Damages</div>
                <div class="metric-value">${:,.0}</div>
            </div>
            <div class="metric-box">
                <div class="metric-label">Recommended Demand</div>
                <div class="metric-value">${:,.0}</div>
            </div>
            <div class="metric-box">
                <div class="metric-label">Target Settlement</div>
                <div class="metric-value">${:,.0}</div>
            </div>
            <div class="metric-box">
                <div class="metric-label">Net to Client</div>
                <div class="metric-value">${:,.0}</div>
            </div>
        </div>
    </div>

    <!-- Settlement Range -->
    <div class="section">
        <h2 class="section-title">Settlement Range</h2>
        <div class="highlight-box">
            <p><strong>Low Estimate:</strong> ${:,.0}</p>
            <p><strong>Mid Estimate:</strong> ${:,.0}</p>
            <p><strong>High Estimate:</strong> ${:,.0}</p>
            <p><strong>Confidence Level:</strong> {:.1}%</p>
            <p style="margin-top: 15px; font-style: italic;">{}</p>
        </div>
    </div>

    <!-- Damages Breakdown -->
    <div class="section">
        <h2 class="section-title">Damages Breakdown</h2>
        <table>
            <thead>
                <tr>
                    <th>Category</th>
                    <th style="text-align: right;">Amount</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td><strong>Economic Damages</strong></td>
                    <td style="text-align: right; font-weight: bold;">${:,.0}</td>
                </tr>
                <tr>
                    <td style="padding-left: 30px;">Past Medical Expenses</td>
                    <td style="text-align: right;">${:,.0}</td>
                </tr>
                <tr>
                    <td style="padding-left: 30px;">Future Medical Expenses</td>
                    <td style="text-align: right;">${:,.0}</td>
                </tr>
                <tr>
                    <td style="padding-left: 30px;">Past Lost Wages</td>
                    <td style="text-align: right;">${:,.0}</td>
                </tr>
                <tr>
                    <td style="padding-left: 30px;">Future Lost Earning Capacity</td>
                    <td style="text-align: right;">${:,.0}</td>
                </tr>
                <tr>
                    <td style="padding-left: 30px;">Other Economic Losses</td>
                    <td style="text-align: right;">${:,.0}</td>
                </tr>
                <tr style="background: #e8f4f8;">
                    <td><strong>Non-Economic Damages</strong></td>
                    <td style="text-align: right; font-weight: bold;">${:,.0}</td>
                </tr>
                <tr>
                    <td style="padding-left: 30px;">Pain and Suffering</td>
                    <td style="text-align: right;">${:,.0}</td>
                </tr>
                <tr>
                    <td style="padding-left: 30px;">Emotional Distress</td>
                    <td style="text-align: right;">${:,.0}</td>
                </tr>
                <tr>
                    <td style="padding-left: 30px;">Loss of Enjoyment of Life</td>
                    <td style="text-align: right;">${:,.0}</td>
                </tr>
                <tr style="background: #1e3a5f; color: white;">
                    <td><strong>TOTAL DAMAGES</strong></td>
                    <td style="text-align: right; font-weight: bold;">${:,.0}</td>
                </tr>
            </tbody>
        </table>
    </div>

    <!-- Liability Analysis -->
    <div class="section">
        <h2 class="section-title">Liability Analysis</h2>
        <p><strong>Defendant Liability:</strong> {:.0}%</p>
        <p><strong>Liability Strength:</strong> {:?}</p>
        <p><strong>Comparative Negligence:</strong> {}</p>
    </div>

    <!-- Risk Assessment -->
    <div class="section">
        <h2 class="section-title">Risk Assessment</h2>
        <div class="metric-grid">
            <div class="metric-box">
                <div class="metric-label">Probability of Win</div>
                <div class="metric-value">{:.0}%</div>
            </div>
            <div class="metric-box">
                <div class="metric-label">Expected Trial Value</div>
                <div class="metric-value">${:,.0}</div>
            </div>
            <div class="metric-box">
                <div class="metric-label">Trial Cost Estimate</div>
                <div class="metric-value">${:,.0}</div>
            </div>
            <div class="metric-box">
                <div class="metric-label">Trial Duration</div>
                <div class="metric-value">{} mos</div>
            </div>
        </div>
    </div>

    <!-- Negotiation Strategy -->
    <div class="section">
        <h2 class="section-title">Negotiation Strategy</h2>
        <ol class="strategy-list" style="counter-reset: strategy;">
            {}
        </ol>
    </div>

    <!-- Rationale -->
    <div class="section">
        <h2 class="section-title">Settlement Rationale</h2>
        <p style="text-align: justify;">{}</p>
    </div>

    <!-- Footer -->
    <div class="footer">
        <p>Settlement Analysis Report | Generated by PA eDocket Settlement Calculator v{} | Confidential Attorney Work Product</p>
    </div>
</body>
</html>"#,
            calc.plaintiff_name,
            calc.defendant_name,
            calc.matter_id,
            calc.liability_analysis.jurisdiction,
            calc.calculated_at.format("%B %d, %Y"),
            calc.calculated_by,
            calc.total_damages,
            calc.recommended_demand,
            calc.target_settlement,
            calc.net_to_client,
            calc.settlement_range.low_estimate,
            calc.settlement_range.mid_estimate,
            calc.settlement_range.high_estimate,
            calc.settlement_range.confidence_level * 100.0,
            calc.settlement_range.range_explanation,
            calc.economic_damages.total_economic,
            calc.economic_damages.past_medical_expenses,
            calc.economic_damages.future_medical_expenses,
            calc.economic_damages.past_lost_wages,
            calc.economic_damages.future_lost_earning_capacity,
            calc.economic_damages.property_damage + calc.economic_damages.other_expenses,
            calc.non_economic_damages.total_non_economic,
            calc.non_economic_damages.pain_and_suffering,
            calc.non_economic_damages.emotional_distress,
            calc.non_economic_damages.loss_of_enjoyment_of_life,
            calc.total_damages,
            calc.liability_analysis.defendant_liability_percentage,
            calc.liability_analysis.liability_strength,
            if calc.liability_analysis.comparative_negligence_applies {
                "Applies"
            } else {
                "Does Not Apply"
            },
            calc.risk_assessment.probability_of_win * 100.0,
            calc.risk_assessment.expected_trial_value,
            calc.risk_assessment.trial_cost_estimate,
            calc.risk_assessment.expected_trial_duration_months,
            calc.negotiation_strategy
                .iter()
                .map(|s| format!("<li>{}</li>", s))
                .collect::<Vec<_>>()
                .join("\n            "),
            calc.rationale,
            calc.version,
        );

        Ok(html)
    }

    // ============= EXCEL GENERATION =============

    /// Generate Excel workbook with settlement analysis
    pub async fn generate_excel_report(
        &self,
        calculation: &SettlementCalculation,
        output_path: &str,
    ) -> Result<PathBuf> {
        // In production, this would use rust_xlsxwriter or similar
        // For now, generate CSV as placeholder

        let csv = self.generate_csv_report(calculation)?;

        let csv_path = PathBuf::from(output_path.replace(".xlsx", ".csv"));
        std::fs::write(&csv_path, csv)?;

        Ok(csv_path)
    }

    fn generate_csv_report(&self, calc: &SettlementCalculation) -> Result<String> {
        let mut csv = String::new();

        // Header
        csv.push_str("SETTLEMENT ANALYSIS REPORT\n");
        csv.push_str(&format!("{} v. {}\n", calc.plaintiff_name, calc.defendant_name));
        csv.push_str(&format!("Matter ID: {}\n", calc.matter_id));
        csv.push_str(&format!("Calculated: {}\n\n", calc.calculated_at.format("%Y-%m-%d")));

        // Summary
        csv.push_str("SUMMARY\n");
        csv.push_str("Metric,Amount\n");
        csv.push_str(&format!("Total Damages,{}\n", calc.total_damages));
        csv.push_str(&format!("Recommended Demand,{}\n", calc.recommended_demand));
        csv.push_str(&format!("Target Settlement,{}\n", calc.target_settlement));
        csv.push_str(&format!("Minimum Settlement,{}\n", calc.minimum_settlement));
        csv.push_str(&format!("Net to Client,{}\n\n", calc.net_to_client));

        // Economic Damages
        csv.push_str("ECONOMIC DAMAGES\n");
        csv.push_str("Category,Amount\n");
        csv.push_str(&format!(
            "Past Medical Expenses,{}\n",
            calc.economic_damages.past_medical_expenses
        ));
        csv.push_str(&format!(
            "Future Medical Expenses,{}\n",
            calc.economic_damages.future_medical_expenses
        ));
        csv.push_str(&format!(
            "Past Lost Wages,{}\n",
            calc.economic_damages.past_lost_wages
        ));
        csv.push_str(&format!(
            "Future Lost Earning Capacity,{}\n",
            calc.economic_damages.future_lost_earning_capacity
        ));
        csv.push_str(&format!(
            "Total Economic,{}\n\n",
            calc.economic_damages.total_economic
        ));

        // Non-Economic Damages
        csv.push_str("NON-ECONOMIC DAMAGES\n");
        csv.push_str("Category,Amount\n");
        csv.push_str(&format!(
            "Pain and Suffering,{}\n",
            calc.non_economic_damages.pain_and_suffering
        ));
        csv.push_str(&format!(
            "Emotional Distress,{}\n",
            calc.non_economic_damages.emotional_distress
        ));
        csv.push_str(&format!(
            "Loss of Enjoyment of Life,{}\n",
            calc.non_economic_damages.loss_of_enjoyment_of_life
        ));
        csv.push_str(&format!(
            "Total Non-Economic,{}\n\n",
            calc.non_economic_damages.total_non_economic
        ));

        // Comparable Verdicts
        csv.push_str("COMPARABLE VERDICTS\n");
        csv.push_str("Case Name,Year,Jurisdiction,Verdict Amount,Similarity\n");
        for verdict in &calc.comparable_verdicts {
            csv.push_str(&format!(
                "\"{}\",{},{},{},{:.0}%\n",
                verdict.case_name,
                verdict.year,
                verdict.jurisdiction,
                verdict.verdict_amount,
                verdict.similarity_score * 100.0
            ));
        }

        Ok(csv)
    }

    // ============= WORD GENERATION =============

    /// Generate Word document
    pub async fn generate_word_report(
        &self,
        calculation: &SettlementCalculation,
        output_path: &str,
    ) -> Result<PathBuf> {
        // In production, this would use docx-rs or similar
        // For now, generate markdown that can be converted

        let markdown = self.generate_markdown_report(calculation)?;

        let md_path = PathBuf::from(output_path.replace(".docx", ".md"));
        std::fs::write(&md_path, markdown)?;

        Ok(md_path)
    }

    fn generate_markdown_report(&self, calc: &SettlementCalculation) -> Result<String> {
        let mut md = String::new();

        md.push_str(&format!("# Settlement Analysis Report\n\n"));
        md.push_str(&format!("## {} v. {}\n\n", calc.plaintiff_name, calc.defendant_name));
        md.push_str(&format!("**Matter ID:** {}\n\n", calc.matter_id));
        md.push_str(&format!("**Calculated:** {}\n\n", calc.calculated_at.format("%B %d, %Y")));

        md.push_str("---\n\n");

        md.push_str("## Executive Summary\n\n");
        md.push_str(&format!("- **Total Damages:** ${:,.0}\n", calc.total_damages));
        md.push_str(&format!("- **Recommended Demand:** ${:,.0}\n", calc.recommended_demand));
        md.push_str(&format!("- **Target Settlement:** ${:,.0}\n", calc.target_settlement));
        md.push_str(&format!("- **Net to Client:** ${:,.0}\n\n", calc.net_to_client));

        md.push_str("## Settlement Range\n\n");
        md.push_str(&format!("- **Low:** ${:,.0}\n", calc.settlement_range.low_estimate));
        md.push_str(&format!("- **Mid:** ${:,.0}\n", calc.settlement_range.mid_estimate));
        md.push_str(&format!("- **High:** ${:,.0}\n", calc.settlement_range.high_estimate));
        md.push_str(&format!("- **Confidence:** {:.1}%\n\n", calc.settlement_range.confidence_level * 100.0));

        md.push_str("## Negotiation Strategy\n\n");
        for (i, strategy) in calc.negotiation_strategy.iter().enumerate() {
            md.push_str(&format!("{}. {}\n", i + 1, strategy));
        }
        md.push_str("\n");

        md.push_str("## Rationale\n\n");
        md.push_str(&format!("{}\n\n", calc.rationale));

        Ok(md)
    }
}
