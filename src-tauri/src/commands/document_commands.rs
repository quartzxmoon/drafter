// Tauri Commands for Document Editor and Citation Features
// Connects the frontend DocumentEditor to backend services

use crate::domain::case_management::*;
use crate::services::ai_citation_service::{AICitationService, CitationSuggestion, ExtractedCitation};
use crate::services::case_management::CaseManagementService;
use crate::services::pleading_formatter::PleadingFormatter;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

// ============================================================================
// Command State
// ============================================================================

pub struct AppState {
    pub db_pool: Pool<Sqlite>,
    pub citation_service: Arc<Mutex<AICitationService>>,
    pub case_service: Arc<Mutex<CaseManagementService>>,
    pub pleading_formatter: Arc<Mutex<PleadingFormatter>>,
}

// ============================================================================
// Document Commands
// ============================================================================

#[tauri::command]
pub async fn cmd_save_document(
    document_id: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Save document content to database
    sqlx::query!(
        r#"UPDATE case_documents SET file_path = ?, updated_at = datetime('now') WHERE id = ?"#,
        content,
        document_id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn cmd_export_document(
    document_id: String,
    content: String,
    format: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Export to PDF or DOCX
    let output_path = match format.as_str() {
        "pdf" => {
            // TODO: Implement PDF generation
            format!("/tmp/{}.pdf", document_id)
        }
        "docx" => {
            // TODO: Implement DOCX generation
            format!("/tmp/{}.docx", document_id)
        }
        _ => return Err("Unsupported format".to_string()),
    };

    // Write content
    std::fs::write(&output_path, content).map_err(|e| e.to_string())?;

    Ok(output_path)
}

// ============================================================================
// Citation Commands
// ============================================================================

#[tauri::command]
pub async fn cmd_search_case_law(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<CitationSuggestion>, String> {
    let service = state.citation_service.lock().await;

    service
        .suggest_citations("", &query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_extract_citations(
    text: String,
    state: State<'_, AppState>,
) -> Result<Vec<ExtractedCitation>, String> {
    let service = state.citation_service.lock().await;

    service
        .extract_citations(&text)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_format_citations(
    text: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let service = state.citation_service.lock().await;

    service
        .format_citations_in_text(&text)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_generate_toa(
    content: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let service = state.citation_service.lock().await;

    service
        .generate_table_of_authorities(&content)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// Pleading Formatting Commands
// ============================================================================

#[tauri::command]
pub async fn cmd_format_as_pleading(
    matter_id: String,
    content: String,
    document_type: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let case_service = state.case_service.lock().await;
    let formatter = state.pleading_formatter.lock().await;

    // Get matter data
    let matter_summary = case_service
        .get_matter_summary(&matter_id)
        .await
        .map_err(|e| e.to_string())?;

    // Get court rules (placeholder - would load from config)
    let court_rules = crate::domain::CourtRules {
        court_id: matter_summary.matter.court_name.clone().unwrap_or_default(),
        margins: crate::domain::CourtMargins {
            top: "1.0in".to_string(),
            bottom: "1.0in".to_string(),
            left: "1.5in".to_string(),
            right: "1.0in".to_string(),
        },
        font: crate::domain::CourtFont {
            family: "Times New Roman".to_string(),
            size: "12pt".to_string(),
            line_spacing: "2.0".to_string(),
        },
        caption: crate::domain::CourtCaption {
            format: "standard".to_string(),
            include_docket: true,
            include_court: true,
            include_county: true,
            include_judge: false,
            include_division: Some(false),
        },
        signature: crate::domain::CourtSignature {
            attorney_name: true,
            attorney_id: true,
            firm_name: true,
            address: true,
            phone: true,
            email: true,
        },
        service_certificate: true,
        table_of_contents: Some(false),
        table_of_authorities: Some(false),
        page_limits: std::collections::HashMap::new(),
    };

    // Parse document type
    let doc_type = serde_json::from_str::<DocumentType>(&format!("\"{}\"", document_type))
        .unwrap_or(DocumentType::Motion);

    // Format document
    let formatted = formatter
        .format_pleading(
            &content,
            &matter_summary.matter,
            &matter_summary.client,
            &doc_type,
            &court_rules,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(formatted.html)
}

// ============================================================================
// Case Management Commands
// ============================================================================

#[tauri::command]
pub async fn cmd_list_matters(
    folder_id: Option<String>,
    practice_area_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<Matter>, String> {
    let service = state.case_service.lock().await;

    service
        .list_matters(None, None)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_get_matter_summary(
    matter_id: String,
    state: State<'_, AppState>,
) -> Result<MatterSummary, String> {
    let service = state.case_service.lock().await;

    service
        .get_matter_summary(&matter_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_create_client(
    request: CreateClientRequest,
    state: State<'_, AppState>,
) -> Result<Client, String> {
    let service = state.case_service.lock().await;

    service
        .create_client(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_create_matter(
    request: CreateMatterRequest,
    state: State<'_, AppState>,
) -> Result<Matter, String> {
    let service = state.case_service.lock().await;

    service
        .create_matter(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_generate_document(
    request: GenerateDocumentRequest,
    state: State<'_, AppState>,
) -> Result<GenerateDocumentResponse, String> {
    let service = state.case_service.lock().await;

    service
        .generate_document(request)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// Folder & Organization Commands
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CaseFolder {
    pub id: String,
    pub name: String,
    pub color: String,
    pub icon: String,
    pub matter_count: i32,
    pub parent_folder_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PracticeArea {
    pub id: String,
    pub name: String,
    pub parent_area_id: Option<String>,
    pub matter_count: i32,
}

#[tauri::command]
pub async fn cmd_get_case_folders(
    state: State<'_, AppState>,
) -> Result<Vec<CaseFolder>, String> {
    let rows = sqlx::query!(
        r#"
        SELECT
            f.id,
            f.name,
            f.color,
            f.icon,
            f.parent_folder_id,
            COUNT(mf.matter_id) as "matter_count!"
        FROM case_folders f
        LEFT JOIN matter_folders mf ON f.id = mf.folder_id
        GROUP BY f.id
        ORDER BY f.sort_order
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| e.to_string())?;

    let folders = rows
        .into_iter()
        .map(|row| CaseFolder {
            id: row.id,
            name: row.name,
            color: row.color.unwrap_or_default(),
            icon: row.icon.unwrap_or_default(),
            matter_count: row.matter_count,
            parent_folder_id: row.parent_folder_id,
        })
        .collect();

    Ok(folders)
}

#[tauri::command]
pub async fn cmd_get_practice_areas(
    state: State<'_, AppState>,
) -> Result<Vec<PracticeArea>, String> {
    let rows = sqlx::query!(
        r#"
        SELECT
            pa.id,
            pa.name,
            pa.parent_area_id,
            COUNT(mpa.matter_id) as "matter_count!"
        FROM practice_areas pa
        LEFT JOIN matter_practice_areas mpa ON pa.id = mpa.practice_area_id
        GROUP BY pa.id
        ORDER BY pa.sort_order
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| e.to_string())?;

    let areas = rows
        .into_iter()
        .map(|row| PracticeArea {
            id: row.id,
            name: row.name,
            parent_area_id: row.parent_area_id,
            matter_count: row.matter_count,
        })
        .collect();

    Ok(areas)
}

// ============================================================================
// AI Assistant Commands
// ============================================================================

#[tauri::command]
pub async fn cmd_get_ai_suggestions(
    matter_id: String,
    context: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    // Placeholder for AI suggestions
    // In production, this would call an AI service
    Ok(vec![
        "Consider adding a citation to support this argument.".to_string(),
        "This section may benefit from additional factual support.".to_string(),
        "Review Pa. R.C.P. 1035.2 for summary judgment standards.".to_string(),
    ])
}

#[tauri::command]
pub async fn cmd_analyze_document(
    content: String,
    document_type: String,
    state: State<'_, AppState>,
) -> Result<DocumentAnalysis, String> {
    let citation_service = state.citation_service.lock().await;

    let citations = citation_service
        .extract_citations(&content)
        .await
        .map_err(|e| e.to_string())?;

    let word_count = content.split_whitespace().count();
    let paragraph_count = content.split("\n\n").count();

    Ok(DocumentAnalysis {
        word_count,
        paragraph_count,
        citation_count: citations.len(),
        has_toa: content.contains("TABLE OF AUTHORITIES"),
        completeness_score: 0.85, // Placeholder
        suggestions: vec![
            "Document structure looks good.".to_string(),
        ],
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentAnalysis {
    pub word_count: usize,
    pub paragraph_count: usize,
    pub citation_count: usize,
    pub has_toa: bool,
    pub completeness_score: f32,
    pub suggestions: Vec<String>,
}
