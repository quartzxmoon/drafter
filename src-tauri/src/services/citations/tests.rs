// Tests for citation engine

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::domain::*;
    use chrono::{DateTime, Utc};

    fn create_test_case() -> Case {
        Case {
            id: "test-case-1".to_string(),
            docket_number: "CP-51-CR-0001234-2023".to_string(),
            court: "Philadelphia County Court of Common Pleas".to_string(),
            case_type: "Criminal".to_string(),
            status: "Active".to_string(),
            filing_date: Some("2023-01-15".to_string()),
            parties: vec![
                Party {
                    id: "party-1".to_string(),
                    name: "Commonwealth of Pennsylvania".to_string(),
                    party_type: "Plaintiff".to_string(),
                    role: "Prosecutor".to_string(),
                    address: None,
                    attorney: None,
                    status: "Active".to_string(),
                },
                Party {
                    id: "party-2".to_string(),
                    name: "John Doe".to_string(),
                    party_type: "Defendant".to_string(),
                    role: "Defendant".to_string(),
                    address: None,
                    attorney: Some(Attorney {
                        id: "atty-1".to_string(),
                        name: "Jane Smith, Esq.".to_string(),
                        bar_number: Some("PA12345".to_string()),
                        firm: Some("Smith & Associates".to_string()),
                        address: None,
                        phone: None,
                        email: None,
                    }),
                    status: "Active".to_string(),
                },
            ],
            charges: vec![],
            events: vec![],
            filings: vec![],
            financials: vec![],
            attachments: vec![],
            judge: Some("Hon. Mary Johnson".to_string()),
            division: Some("Criminal Division".to_string()),
            last_updated: Some("2023-12-01T10:00:00Z".to_string()),
            source: "UJS Portal".to_string(),
        }
    }

    fn create_test_filing() -> Filing {
        Filing {
            id: "filing-1".to_string(),
            filing_date: "2023-01-20".to_string(),
            document_type: "Motion".to_string(),
            description: "Motion to Suppress Evidence".to_string(),
            filed_by: "Jane Smith, Esq.".to_string(),
            status: "Filed".to_string(),
            pages: Some(15),
            attachments: vec![],
        }
    }

    #[test]
    fn test_citation_parser_case_citation() {
        let parser = CitationParser::new();
        
        let text = "Commonwealth v. Doe, 123 Pa. 456 (2023)";
        let citations = parser.parse_citations(text);
        
        assert_eq!(citations.len(), 1);
        let citation = &citations[0];
        assert_eq!(citation.citation_type, CitationType::Case);
        assert_eq!(citation.case_name, Some("Commonwealth v. Doe".to_string()));
        assert_eq!(citation.volume, Some("123".to_string()));
        assert_eq!(citation.reporter, Some("Pa.".to_string()));
        assert_eq!(citation.page, Some("456".to_string()));
        assert_eq!(citation.year, Some("2023".to_string()));
    }

    #[test]
    fn test_citation_parser_statute_citation() {
        let parser = CitationParser::new();
        
        let text = "18 Pa.C.S. § 3502";
        let citations = parser.parse_citations(text);
        
        assert_eq!(citations.len(), 1);
        let citation = &citations[0];
        assert_eq!(citation.citation_type, CitationType::Statute);
        assert_eq!(citation.title, Some("18".to_string()));
        assert_eq!(citation.code, Some("Pa.C.S.".to_string()));
        assert_eq!(citation.section, Some("3502".to_string()));
    }

    #[test]
    fn test_citation_parser_rule_citation() {
        let parser = CitationParser::new();
        
        let text = "Pa.R.Crim.P. 600";
        let citations = parser.parse_citations(text);
        
        assert_eq!(citations.len(), 1);
        let citation = &citations[0];
        assert_eq!(citation.citation_type, CitationType::Rule);
        assert_eq!(citation.rule_set, Some("Pa.R.Crim.P.".to_string()));
        assert_eq!(citation.rule_number, Some("600".to_string()));
    }

    #[test]
    fn test_citation_formatter_case_citation() {
        let formatter = CitationFormatter::new();
        
        let citation = Citation {
            id: "cite-1".to_string(),
            citation_type: CitationType::Case,
            case_name: Some("Commonwealth v. Doe".to_string()),
            volume: Some("123".to_string()),
            reporter: Some("Pa.".to_string()),
            page: Some("456".to_string()),
            year: Some("2023".to_string()),
            ..Default::default()
        };
        
        let formatted = formatter.format_citation(&citation, &CitationStyle::Bluebook);
        assert_eq!(formatted, "Commonwealth v. Doe, 123 Pa. 456 (2023)");
    }

    #[test]
    fn test_citation_formatter_statute_citation() {
        let formatter = CitationFormatter::new();
        
        let citation = Citation {
            id: "cite-2".to_string(),
            citation_type: CitationType::Statute,
            title: Some("18".to_string()),
            code: Some("Pa.C.S.".to_string()),
            section: Some("3502".to_string()),
            ..Default::default()
        };
        
        let formatted = formatter.format_citation(&citation, &CitationStyle::Bluebook);
        assert_eq!(formatted, "18 Pa.C.S. § 3502");
    }

    #[test]
    fn test_citation_validator_valid_case() {
        let validator = CitationValidator::new();
        
        let citation = Citation {
            id: "cite-1".to_string(),
            citation_type: CitationType::Case,
            case_name: Some("Commonwealth v. Doe".to_string()),
            volume: Some("123".to_string()),
            reporter: Some("Pa.".to_string()),
            page: Some("456".to_string()),
            year: Some("2023".to_string()),
            ..Default::default()
        };
        
        let result = validator.validate_citation(&citation);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_citation_validator_invalid_case() {
        let validator = CitationValidator::new();
        
        let citation = Citation {
            id: "cite-1".to_string(),
            citation_type: CitationType::Case,
            case_name: None, // Missing required field
            volume: Some("123".to_string()),
            reporter: Some("Pa.".to_string()),
            page: Some("456".to_string()),
            year: Some("2023".to_string()),
            ..Default::default()
        };
        
        let result = validator.validate_citation(&citation);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        assert!(result.errors.iter().any(|e| e.contains("case_name")));
    }

    #[test]
    fn test_citation_engine_generate_case_citation() {
        let engine = CitationEngine::new();
        let case = create_test_case();
        
        let citation = engine.generate_case_citation(&case);
        
        assert_eq!(citation.citation_type, CitationType::Case);
        assert_eq!(citation.case_name, Some("Commonwealth v. Doe".to_string()));
        assert_eq!(citation.docket_number, Some("CP-51-CR-0001234-2023".to_string()));
        assert_eq!(citation.court, Some("Philadelphia County Court of Common Pleas".to_string()));
        assert_eq!(citation.year, Some("2023".to_string()));
    }

    #[test]
    fn test_citation_engine_generate_filing_citation() {
        let engine = CitationEngine::new();
        let case = create_test_case();
        let filing = create_test_filing();
        
        let citation = engine.generate_filing_citation(&case, &filing);
        
        assert_eq!(citation.citation_type, CitationType::Filing);
        assert_eq!(citation.case_name, Some("Commonwealth v. Doe".to_string()));
        assert_eq!(citation.docket_number, Some("CP-51-CR-0001234-2023".to_string()));
        assert_eq!(citation.document_type, Some("Motion".to_string()));
        assert_eq!(citation.filing_date, Some("2023-01-20".to_string()));
    }

    #[test]
    fn test_citation_engine_multiple_citations() {
        let engine = CitationEngine::new();
        
        let text = "See Commonwealth v. Smith, 100 Pa. 200 (2020); 18 Pa.C.S. § 3502; Pa.R.Crim.P. 600.";
        let citations = engine.extract_citations(text);
        
        assert_eq!(citations.len(), 3);
        
        // Check case citation
        let case_citation = citations.iter().find(|c| c.citation_type == CitationType::Case).unwrap();
        assert_eq!(case_citation.case_name, Some("Commonwealth v. Smith".to_string()));
        
        // Check statute citation
        let statute_citation = citations.iter().find(|c| c.citation_type == CitationType::Statute).unwrap();
        assert_eq!(statute_citation.section, Some("3502".to_string()));
        
        // Check rule citation
        let rule_citation = citations.iter().find(|c| c.citation_type == CitationType::Rule).unwrap();
        assert_eq!(rule_citation.rule_number, Some("600".to_string()));
    }

    #[test]
    fn test_citation_style_formats() {
        let formatter = CitationFormatter::new();
        
        let citation = Citation {
            id: "cite-1".to_string(),
            citation_type: CitationType::Case,
            case_name: Some("Commonwealth v. Doe".to_string()),
            volume: Some("123".to_string()),
            reporter: Some("Pa.".to_string()),
            page: Some("456".to_string()),
            year: Some("2023".to_string()),
            ..Default::default()
        };
        
        // Test different citation styles
        let bluebook = formatter.format_citation(&citation, &CitationStyle::Bluebook);
        let alwd = formatter.format_citation(&citation, &CitationStyle::ALWD);
        let chicago = formatter.format_citation(&citation, &CitationStyle::Chicago);
        
        assert_eq!(bluebook, "Commonwealth v. Doe, 123 Pa. 456 (2023)");
        assert_eq!(alwd, "Commonwealth v. Doe, 123 Pa. 456 (2023)");
        assert_eq!(chicago, "Commonwealth v. Doe, 123 Pa. 456 (2023)");
    }

    #[test]
    fn test_citation_normalization() {
        let parser = CitationParser::new();
        
        // Test various formats of the same citation
        let variations = vec![
            "Commonwealth v. Doe, 123 Pa. 456 (2023)",
            "Commonwealth v. Doe, 123 Pa 456 (2023)",
            "Commonwealth v. Doe 123 Pa. 456 (2023)",
            "Commonwealth v. Doe, 123 Pa. 456(2023)",
        ];
        
        for variation in variations {
            let citations = parser.parse_citations(variation);
            assert_eq!(citations.len(), 1);
            let citation = &citations[0];
            assert_eq!(citation.case_name, Some("Commonwealth v. Doe".to_string()));
            assert_eq!(citation.volume, Some("123".to_string()));
            assert_eq!(citation.reporter, Some("Pa.".to_string()));
            assert_eq!(citation.page, Some("456".to_string()));
            assert_eq!(citation.year, Some("2023".to_string()));
        }
    }

    #[test]
    fn test_citation_edge_cases() {
        let parser = CitationParser::new();
        
        // Test empty string
        let citations = parser.parse_citations("");
        assert_eq!(citations.len(), 0);
        
        // Test string with no citations
        let citations = parser.parse_citations("This is just regular text with no citations.");
        assert_eq!(citations.len(), 0);
        
        // Test malformed citations
        let citations = parser.parse_citations("Commonwealth v. 123 Pa. (2023)"); // Missing page
        assert_eq!(citations.len(), 0);
        
        // Test partial citations
        let citations = parser.parse_citations("123 Pa. 456"); // Missing case name and year
        assert_eq!(citations.len(), 0);
    }

    #[test]
    fn test_citation_performance() {
        let parser = CitationParser::new();
        
        // Create a large text with many citations
        let mut large_text = String::new();
        for i in 1..=100 {
            large_text.push_str(&format!("Commonwealth v. Defendant{}, {} Pa. {} (2023). ", i, i, i * 10));
        }
        
        let start = std::time::Instant::now();
        let citations = parser.parse_citations(&large_text);
        let duration = start.elapsed();
        
        assert_eq!(citations.len(), 100);
        assert!(duration.as_millis() < 1000); // Should complete in under 1 second
    }

    // Property-based testing helpers
    use proptest::prelude::*;

    prop_compose! {
        fn arb_case_name()(
            plaintiff in "[A-Z][a-z]{2,10}",
            defendant in "[A-Z][a-z]{2,10}"
        ) -> String {
            format!("{} v. {}", plaintiff, defendant)
        }
    }

    prop_compose! {
        fn arb_volume()(vol in 1u32..1000) -> String {
            vol.to_string()
        }
    }

    prop_compose! {
        fn arb_page()(page in 1u32..9999) -> String {
            page.to_string()
        }
    }

    prop_compose! {
        fn arb_year()(year in 1900u32..2030) -> String {
            year.to_string()
        }
    }

    proptest! {
        #[test]
        fn test_citation_roundtrip(
            case_name in arb_case_name(),
            volume in arb_volume(),
            page in arb_page(),
            year in arb_year()
        ) {
            let parser = CitationParser::new();
            let formatter = CitationFormatter::new();
            
            let original_citation = Citation {
                id: "test".to_string(),
                citation_type: CitationType::Case,
                case_name: Some(case_name.clone()),
                volume: Some(volume.clone()),
                reporter: Some("Pa.".to_string()),
                page: Some(page.clone()),
                year: Some(year.clone()),
                ..Default::default()
            };
            
            let formatted = formatter.format_citation(&original_citation, &CitationStyle::Bluebook);
            let parsed_citations = parser.parse_citations(&formatted);
            
            prop_assert_eq!(parsed_citations.len(), 1);
            let parsed_citation = &parsed_citations[0];
            prop_assert_eq!(parsed_citation.case_name, Some(case_name));
            prop_assert_eq!(parsed_citation.volume, Some(volume));
            prop_assert_eq!(parsed_citation.page, Some(page));
            prop_assert_eq!(parsed_citation.year, Some(year));
        }
    }
}
