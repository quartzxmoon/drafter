// Export service for PA eDocket Desktop

use crate::domain::*;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;
use zip::{write::FileOptions, ZipWriter};

pub struct ExportService {
    output_dir: PathBuf,
    temp_dir: PathBuf,
}

impl ExportService {
    pub fn new(output_dir: PathBuf) -> Self {
        let temp_dir = output_dir.join("temp");
        Self {
            output_dir,
            temp_dir,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        fs::create_dir_all(&self.output_dir)?;
        fs::create_dir_all(&self.temp_dir)?;
        info!("Export service initialized with output dir: {:?}", self.output_dir);
        Ok(())
    }

    #[instrument(skip(self, data))]
    pub async fn export_json(&self, data: &serde_json::Value, output_path: &str) -> Result<ExportManifest> {
        info!("Exporting data to JSON: {}", output_path);

        let full_path = self.resolve_output_path(output_path)?;

        // Write JSON data
        let json_content = serde_json::to_string_pretty(data)?;
        fs::write(&full_path, &json_content)?;

        // Calculate hash
        let hash = self.calculate_file_hash(&full_path)?;

        // Create manifest
        let manifest = ExportManifest {
            id: Uuid::new_v4(),
            export_type: ExportType::Json,
            created_at: Utc::now(),
            files: vec![ExportFile {
                path: full_path.to_string_lossy().to_string(),
                filename: full_path.file_name().unwrap().to_string_lossy().to_string(),
                size: json_content.len() as u64,
                hash,
                content_type: "application/json".to_string(),
            }],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("record_count".to_string(), self.count_json_records(data).to_string());
                meta.insert("format_version".to_string(), "1.0".to_string());
                meta
            },
            audit_trail: vec![AuditEntry {
                timestamp: Utc::now(),
                action: "export_created".to_string(),
                user: "system".to_string(),
                details: format!("JSON export to {}", output_path),
            }],
        };

        self.save_manifest(&manifest).await?;

        info!("JSON export completed: {} bytes", json_content.len());
        Ok(manifest)
    }

    #[instrument(skip(self, data))]
    pub async fn export_csv(&self, data: &[SearchResult], output_path: &str) -> Result<ExportManifest> {
        info!("Exporting {} search results to CSV: {}", data.len(), output_path);

        let full_path = self.resolve_output_path(output_path)?;

        // Create CSV content
        let mut csv_content = String::new();

        // Header
        csv_content.push_str("ID,Caption,Court,County,Filed,Status,Docket Number,OTN,SID,Judge,Courtroom,Last Updated\n");

        // Data rows
        for result in data {
            csv_content.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{}\n",
                self.escape_csv(&result.id),
                self.escape_csv(&result.caption),
                self.format_court_level(&result.court),
                self.escape_csv(&result.county),
                self.escape_csv(&result.filed),
                self.format_case_status(&result.status),
                self.escape_csv(&result.docket_number.as_deref().unwrap_or("")),
                self.escape_csv(&result.otn.as_deref().unwrap_or("")),
                self.escape_csv(&result.sid.as_deref().unwrap_or("")),
                self.escape_csv(&result.judge.as_deref().unwrap_or("")),
                self.escape_csv(&result.courtroom.as_deref().unwrap_or("")),
                result.last_updated.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default()
            ));
        }

        // Write CSV file
        fs::write(&full_path, &csv_content)?;

        // Calculate hash
        let hash = self.calculate_file_hash(&full_path)?;

        // Create manifest
        let manifest = ExportManifest {
            id: Uuid::new_v4(),
            export_type: ExportType::Csv,
            created_at: Utc::now(),
            files: vec![ExportFile {
                path: full_path.to_string_lossy().to_string(),
                filename: full_path.file_name().unwrap().to_string_lossy().to_string(),
                size: csv_content.len() as u64,
                hash,
                content_type: "text/csv".to_string(),
            }],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("record_count".to_string(), data.len().to_string());
                meta.insert("format_version".to_string(), "1.0".to_string());
                meta.insert("encoding".to_string(), "UTF-8".to_string());
                meta
            },
            audit_trail: vec![AuditEntry {
                timestamp: Utc::now(),
                action: "export_created".to_string(),
                user: "system".to_string(),
                details: format!("CSV export of {} records to {}", data.len(), output_path),
            }],
        };

        self.save_manifest(&manifest).await?;

        info!("CSV export completed: {} records, {} bytes", data.len(), csv_content.len());
        Ok(manifest)
    }

    #[instrument(skip(self, docket))]
    pub async fn export_pdf(&self, docket: &Docket, output_path: &str) -> Result<ExportManifest> {
        info!("Exporting docket to PDF: {}", output_path);

        let full_path = self.resolve_output_path(output_path)?;

        // Generate HTML content for the docket
        let html_content = self.generate_docket_html(docket)?;

        // For now, save as HTML (in a real implementation, convert to PDF)
        let html_path = full_path.with_extension("html");
        fs::write(&html_path, &html_content)?;

        // Calculate hash
        let hash = self.calculate_file_hash(&html_path)?;

        // Create manifest
        let manifest = ExportManifest {
            id: Uuid::new_v4(),
            export_type: ExportType::Pdf,
            created_at: Utc::now(),
            files: vec![ExportFile {
                path: html_path.to_string_lossy().to_string(),
                filename: html_path.file_name().unwrap().to_string_lossy().to_string(),
                size: html_content.len() as u64,
                hash,
                content_type: "text/html".to_string(), // Would be "application/pdf" for real PDF
            }],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("docket_id".to_string(), docket.id.clone());
                meta.insert("docket_number".to_string(), docket.docket_number.clone());
                meta.insert("case_caption".to_string(), docket.caption.clone());
                meta.insert("party_count".to_string(), docket.parties.len().to_string());
                meta.insert("event_count".to_string(), docket.events.len().to_string());
                meta
            },
            audit_trail: vec![AuditEntry {
                timestamp: Utc::now(),
                action: "export_created".to_string(),
                user: "system".to_string(),
                details: format!("PDF export of docket {} to {}", docket.docket_number, output_path),
            }],
        };

        self.save_manifest(&manifest).await?;

        info!("PDF export completed: {} bytes", html_content.len());
        Ok(manifest)
    }

    #[instrument(skip(self, files))]
    pub async fn create_zip(&self, files: &[String], output_path: &str) -> Result<ExportManifest> {
        info!("Creating ZIP archive with {} files: {}", files.len(), output_path);

        let full_path = self.resolve_output_path(output_path)?;

        // Create ZIP file
        let zip_file = File::create(&full_path)?;
        let mut zip = ZipWriter::new(zip_file);

        let mut total_size = 0u64;
        let mut zip_files = Vec::new();

        for file_path in files {
            let path = Path::new(file_path);
            if !path.exists() {
                warn!("File not found, skipping: {}", file_path);
                continue;
            }

            let file_content = fs::read(path)?;
            let filename = path.file_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid filename: {}", file_path))?
                .to_string_lossy();

            // Add file to ZIP
            zip.start_file(&filename, FileOptions::default())?;
            zip.write_all(&file_content)?;

            total_size += file_content.len() as u64;

            zip_files.push(ExportFile {
                path: file_path.clone(),
                filename: filename.to_string(),
                size: file_content.len() as u64,
                hash: format!("{:x}", Sha256::digest(&file_content)),
                content_type: self.detect_content_type(path),
            });
        }

        // Add manifest to ZIP
        let manifest_content = self.create_zip_manifest(files)?;
        zip.start_file("manifest.json", FileOptions::default())?;
        zip.write_all(manifest_content.as_bytes())?;

        zip.finish()?;

        // Calculate ZIP hash
        let zip_hash = self.calculate_file_hash(&full_path)?;
        let zip_size = fs::metadata(&full_path)?.len();

        // Create export manifest
        let manifest = ExportManifest {
            id: Uuid::new_v4(),
            export_type: ExportType::Zip,
            created_at: Utc::now(),
            files: vec![ExportFile {
                path: full_path.to_string_lossy().to_string(),
                filename: full_path.file_name().unwrap().to_string_lossy().to_string(),
                size: zip_size,
                hash: zip_hash,
                content_type: "application/zip".to_string(),
            }],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("file_count".to_string(), files.len().to_string());
                meta.insert("total_uncompressed_size".to_string(), total_size.to_string());
                meta.insert("compression_ratio".to_string(),
                    format!("{:.2}", (zip_size as f64 / total_size as f64) * 100.0));
                meta
            },
            audit_trail: vec![AuditEntry {
                timestamp: Utc::now(),
                action: "export_created".to_string(),
                user: "system".to_string(),
                details: format!("ZIP archive of {} files to {}", files.len(), output_path),
            }],
        };

        self.save_manifest(&manifest).await?;

        info!("ZIP export completed: {} files, {} bytes compressed", files.len(), zip_size);
        Ok(manifest)
    }

    // Helper methods
    fn resolve_output_path(&self, output_path: &str) -> Result<PathBuf> {
        let path = if Path::new(output_path).is_absolute() {
            PathBuf::from(output_path)
        } else {
            self.output_dir.join(output_path)
        };

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        Ok(path)
    }

    fn calculate_file_hash(&self, path: &Path) -> Result<String> {
        let content = fs::read(path)?;
        let hash = Sha256::digest(&content);
        Ok(format!("{:x}", hash))
    }

    async fn save_manifest(&self, manifest: &ExportManifest) -> Result<()> {
        let manifest_filename = format!("manifest_{}.json", manifest.id);
        let manifest_path = self.output_dir.join(manifest_filename);

        let json_content = serde_json::to_string_pretty(manifest)?;
        fs::write(manifest_path, json_content)?;

        Ok(())
    }

    fn count_json_records(&self, data: &serde_json::Value) -> usize {
        match data {
            serde_json::Value::Array(arr) => arr.len(),
            serde_json::Value::Object(obj) => {
                if let Some(serde_json::Value::Array(arr)) = obj.get("results") {
                    arr.len()
                } else {
                    1
                }
            },
            _ => 1,
        }
    }

    fn escape_csv(&self, value: &str) -> String {
        if value.contains(',') || value.contains('"') || value.contains('\n') {
            format!("\"{}\"", value.replace('"', "\"\""))
        } else {
            value.to_string()
        }
    }

    fn format_court_level(&self, court: &CourtLevel) -> String {
        match court {
            CourtLevel::Mdj => "MDJ".to_string(),
            CourtLevel::Cp => "CP".to_string(),
            CourtLevel::App => "APP".to_string(),
        }
    }

    fn format_case_status(&self, status: &CaseStatus) -> String {
        match status {
            CaseStatus::Active => "Active".to_string(),
            CaseStatus::Closed => "Closed".to_string(),
            CaseStatus::Disposed => "Disposed".to_string(),
        }
    }

    fn detect_content_type(&self, path: &Path) -> String {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => "application/json".to_string(),
            Some("csv") => "text/csv".to_string(),
            Some("pdf") => "application/pdf".to_string(),
            Some("html") => "text/html".to_string(),
            Some("txt") => "text/plain".to_string(),
            Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
            Some("zip") => "application/zip".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    }

    fn create_zip_manifest(&self, files: &[String]) -> Result<String> {
        let manifest = serde_json::json!({
            "created_at": Utc::now(),
            "files": files,
            "total_files": files.len(),
            "created_by": "PA eDocket Desktop",
            "version": "1.0"
        });

        Ok(serde_json::to_string_pretty(&manifest)?)
    }

    fn generate_docket_html(&self, docket: &Docket) -> Result<String> {
        let mut html = String::from("<!DOCTYPE html><html><head><meta charset='utf-8'>");
        html.push_str("<title>Docket Report</title>");
        html.push_str("<style>");
        html.push_str("body { font-family: 'Times New Roman', serif; font-size: 12pt; margin: 1in; }");
        html.push_str("h1 { text-align: center; margin-bottom: 20px; }");
        html.push_str("h2 { border-bottom: 1px solid #000; margin-top: 20px; }");
        html.push_str("table { width: 100%; border-collapse: collapse; margin: 10px 0; }");
        html.push_str("th, td { border: 1px solid #000; padding: 5px; text-align: left; }");
        html.push_str("th { background-color: #f0f0f0; font-weight: bold; }");
        html.push_str(".header-info { margin-bottom: 20px; }");
        html.push_str(".header-info div { margin: 5px 0; }");
        html.push_str("</style></head><body>");

        // Header
        html.push_str(&format!("<h1>DOCKET REPORT</h1>"));
        html.push_str("<div class='header-info'>");
        html.push_str(&format!("<div><strong>Caption:</strong> {}</div>", docket.caption));
        html.push_str(&format!("<div><strong>Docket Number:</strong> {}</div>", docket.docket_number));
        html.push_str(&format!("<div><strong>Court:</strong> {}</div>", docket.court));
        html.push_str(&format!("<div><strong>Status:</strong> {}</div>", self.format_case_status(&docket.status)));
        html.push_str(&format!("<div><strong>Filed:</strong> {}</div>", docket.filed.format("%B %d, %Y")));
        if let Some(judge) = &docket.judge {
            html.push_str(&format!("<div><strong>Judge:</strong> {}</div>", judge));
        }
        html.push_str(&format!("<div><strong>Generated:</strong> {}</div>", Utc::now().format("%B %d, %Y at %H:%M UTC")));
        html.push_str("</div>");

        // Parties
        if !docket.parties.is_empty() {
            html.push_str("<h2>PARTIES</h2>");
            html.push_str("<table>");
            html.push_str("<tr><th>Name</th><th>Role</th><th>Attorney</th></tr>");
            for party in &docket.parties {
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                    party.name,
                    self.format_party_role(&party.role),
                    party.attorney.as_deref().unwrap_or("Pro Se")
                ));
            }
            html.push_str("</table>");
        }

        // Events
        if !docket.events.is_empty() {
            html.push_str("<h2>DOCKET EVENTS</h2>");
            html.push_str("<table>");
            html.push_str("<tr><th>Date</th><th>Description</th><th>Judge</th></tr>");
            for event in &docket.events {
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                    event.date.format("%m/%d/%Y"),
                    event.description,
                    event.judge.as_deref().unwrap_or("")
                ));
            }
            html.push_str("</table>");
        }

        html.push_str("</body></html>");
        Ok(html)
    }

    fn format_party_role(&self, role: &PartyRole) -> String {
        match role {
            PartyRole::Plaintiff => "Plaintiff".to_string(),
            PartyRole::Defendant => "Defendant".to_string(),
            PartyRole::Petitioner => "Petitioner".to_string(),
            PartyRole::Respondent => "Respondent".to_string(),
        }
    }
}

// Data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportManifest {
    pub id: Uuid,
    pub export_type: ExportType,
    pub created_at: DateTime<Utc>,
    pub files: Vec<ExportFile>,
    pub metadata: HashMap<String, String>,
    pub audit_trail: Vec<AuditEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFile {
    pub path: String,
    pub filename: String,
    pub size: u64,
    pub hash: String,
    pub content_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportType {
    Json,
    Csv,
    Pdf,
    Zip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub user: String,
    pub details: String,
}
