// Benchmark tests for citation engine performance

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pa_edocket_desktop::services::citations::CitationService;
use pa_edocket_desktop::domain::*;

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

fn create_test_citation() -> Citation {
    Citation {
        id: "cite-1".to_string(),
        citation_type: CitationType::Case,
        case_name: Some("Commonwealth v. Doe".to_string()),
        volume: Some("123".to_string()),
        reporter: Some("Pa.".to_string()),
        page: Some("456".to_string()),
        year: Some("2023".to_string()),
        ..Default::default()
    }
}

fn generate_large_text_with_citations(num_citations: usize) -> String {
    let mut text = String::new();
    text.push_str("This is a legal document with multiple citations. ");
    
    for i in 1..=num_citations {
        text.push_str(&format!(
            "See Commonwealth v. Defendant{}, {} Pa. {} (2023). ",
            i, 100 + i, 100 + i * 10
        ));
        
        if i % 10 == 0 {
            text.push_str("Additionally, refer to ");
        }
        
        text.push_str(&format!("18 Pa.C.S. § {}. ", 3500 + i));
        
        if i % 5 == 0 {
            text.push_str(&format!("Pa.R.Crim.P. {}. ", 600 + i));
        }
    }
    
    text.push_str("This concludes the document with citations.");
    text
}

async fn bench_parse_citations(text: &str) {
    let service = CitationService::new();
    let _ = service.parse_citations(text, None).await;
}

async fn bench_format_citation(citation: &Citation) {
    let service = CitationService::new();
    let _ = service.format_citation(citation, None).await;
}

async fn bench_validate_citation(citation: &Citation) {
    let service = CitationService::new();
    let _ = service.validate_citation(citation).await;
}

async fn bench_generate_case_citation(case: &Case) {
    let service = CitationService::new();
    let _ = service.generate_case_citation(case).await;
}

fn citation_parsing_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("citation_parsing");
    
    // Test with different numbers of citations
    for num_citations in [1, 10, 50, 100, 500].iter() {
        let text = generate_large_text_with_citations(*num_citations);
        
        group.bench_with_input(
            BenchmarkId::new("parse_citations", num_citations),
            &text,
            |b, text| {
                b.to_async(&rt).iter(|| {
                    bench_parse_citations(black_box(text))
                });
            },
        );
    }
    
    group.finish();
}

fn citation_formatting_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let citation = create_test_citation();
    
    c.bench_function("format_citation", |b| {
        b.to_async(&rt).iter(|| {
            bench_format_citation(black_box(&citation))
        });
    });
}

fn citation_validation_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let citation = create_test_citation();
    
    c.bench_function("validate_citation", |b| {
        b.to_async(&rt).iter(|| {
            bench_validate_citation(black_box(&citation))
        });
    });
}

fn case_citation_generation_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let case = create_test_case();
    
    c.bench_function("generate_case_citation", |b| {
        b.to_async(&rt).iter(|| {
            bench_generate_case_citation(black_box(&case))
        });
    });
}

fn citation_memory_usage_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("citation_memory");
    
    // Test memory usage with large numbers of citations
    for num_citations in [100, 1000, 5000].iter() {
        let text = generate_large_text_with_citations(*num_citations);
        
        group.bench_with_input(
            BenchmarkId::new("large_text_parsing", num_citations),
            &text,
            |b, text| {
                b.to_async(&rt).iter(|| {
                    bench_parse_citations(black_box(text))
                });
            },
        );
    }
    
    group.finish();
}

fn citation_concurrent_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("concurrent_citation_parsing", |b| {
        b.to_async(&rt).iter(|| async {
            let service = CitationService::new();
            let texts = vec![
                "Commonwealth v. Smith, 100 Pa. 200 (2020)",
                "18 Pa.C.S. § 3502",
                "Pa.R.Crim.P. 600",
                "Commonwealth v. Jones, 200 Pa. 300 (2021)",
                "42 Pa.C.S. § 5525",
            ];
            
            let futures: Vec<_> = texts.iter().map(|text| {
                service.parse_citations(text, None)
            }).collect();
            
            let _ = futures::future::join_all(futures).await;
        });
    });
}

fn citation_stress_test_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("citation_stress_test", |b| {
        b.to_async(&rt).iter(|| async {
            let service = CitationService::new();
            
            // Simulate a stress test with many operations
            for i in 0..100 {
                let text = format!("Commonwealth v. Defendant{}, {} Pa. {} (2023)", i, i + 100, i * 10);
                let _ = service.parse_citations(&text, None).await;
                
                let citation = Citation {
                    id: format!("cite-{}", i),
                    citation_type: CitationType::Case,
                    case_name: Some(format!("Commonwealth v. Defendant{}", i)),
                    volume: Some((i + 100).to_string()),
                    reporter: Some("Pa.".to_string()),
                    page: Some((i * 10).to_string()),
                    year: Some("2023".to_string()),
                    ..Default::default()
                };
                
                let _ = service.format_citation(&citation, None).await;
                let _ = service.validate_citation(&citation).await;
            }
        });
    });
}

criterion_group!(
    benches,
    citation_parsing_benchmark,
    citation_formatting_benchmark,
    citation_validation_benchmark,
    case_citation_generation_benchmark,
    citation_memory_usage_benchmark,
    citation_concurrent_benchmark,
    citation_stress_test_benchmark
);

criterion_main!(benches);
