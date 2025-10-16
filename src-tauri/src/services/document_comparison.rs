// Document Comparison Service
// Advanced document comparison with redlining and track changes

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tracing::{info, warn, error};
use similar::{ChangeTag, TextDiff};
use regex::Regex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentComparison {
    pub id: String,
    pub original_document_id: String,
    pub revised_document_id: String,
    pub comparison_type: ComparisonType,
    pub changes: Vec<Change>,
    pub statistics: ComparisonStatistics,
    pub metadata: ComparisonMetadata,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ComparisonType {
    WordLevel,
    LineLevel,
    ParagraphLevel,
    SentenceLevel,
    LegalCitation,
    StructuralOnly,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Change {
    pub id: String,
    pub change_type: ChangeType,
    pub original_text: Option<String>,
    pub revised_text: Option<String>,
    pub position: TextPosition,
    pub confidence: f32,
    pub category: ChangeCategory,
    pub author: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub comment: Option<String>,
    pub accepted: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChangeType {
    Insert,
    Delete,
    Replace,
    Move,
    FormatChange,
    CitationChange,
    NumberingChange,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TextPosition {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub start_offset: u32,
    pub end_offset: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChangeCategory {
    Substantive,
    Editorial,
    Formatting,
    Citation,
    Numbering,
    Metadata,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComparisonStatistics {
    pub total_changes: u32,
    pub insertions: u32,
    pub deletions: u32,
    pub replacements: u32,
    pub moves: u32,
    pub words_added: u32,
    pub words_removed: u32,
    pub characters_added: u32,
    pub characters_removed: u32,
    pub similarity_score: f32,
    pub change_density: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComparisonMetadata {
    pub original_title: String,
    pub revised_title: String,
    pub original_author: Option<String>,
    pub revised_author: Option<String>,
    pub original_date: Option<DateTime<Utc>>,
    pub revised_date: Option<DateTime<Utc>>,
    pub comparison_settings: ComparisonSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComparisonSettings {
    pub ignore_whitespace: bool,
    pub ignore_case: bool,
    pub ignore_punctuation: bool,
    pub ignore_formatting: bool,
    pub ignore_citations: bool,
    pub ignore_numbering: bool,
    pub minimum_change_length: u32,
    pub context_lines: u32,
    pub word_wrap_threshold: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RedlineDocument {
    pub content: String,
    pub format: RedlineFormat,
    pub changes: Vec<Change>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RedlineFormat {
    Html,
    Markdown,
    WordTrackChanges,
    PlainText,
    Pdf,
}

pub struct DocumentComparisonService {
    settings: ComparisonSettings,
    citation_regex: Regex,
    numbering_regex: Regex,
}

impl DocumentComparisonService {
    pub fn new(settings: ComparisonSettings) -> Self {
        let citation_regex = Regex::new(
            r"(?i)\b(?:\d+\s+(?:U\.S\.|F\.\s*(?:2d|3d)?|S\.\s*Ct\.|L\.\s*Ed\.|A\.\s*(?:2d|3d)?|P\.\s*(?:2d|3d)?|N\.E\.\s*(?:2d|3d)?|N\.W\.\s*(?:2d|3d)?|S\.E\.\s*(?:2d|3d)?|S\.W\.\s*(?:2d|3d)?|So\.\s*(?:2d|3d)?)\s+\d+)"
        ).unwrap();

        let numbering_regex = Regex::new(
            r"(?m)^(?:\d+\.|\([a-z]\)|\([0-9]+\)|[IVX]+\.|[a-z]\.|•|\*)"
        ).unwrap();

        Self {
            settings,
            citation_regex,
            numbering_regex,
        }
    }

    pub fn compare_documents(&self, original: &str, revised: &str) -> Result<DocumentComparison> {
        let start_time = std::time::Instant::now();

        // Preprocess documents based on settings
        let processed_original = self.preprocess_text(original);
        let processed_revised = self.preprocess_text(revised);

        // Perform diff analysis
        let diff = TextDiff::from_lines(&processed_original, &processed_revised);
        
        // Extract changes
        let changes = self.extract_changes(&diff, original, revised)?;
        
        // Calculate statistics
        let statistics = self.calculate_statistics(&changes, original, revised);
        
        // Create comparison metadata
        let metadata = ComparisonMetadata {
            original_title: "Original Document".to_string(),
            revised_title: "Revised Document".to_string(),
            original_author: None,
            revised_author: None,
            original_date: None,
            revised_date: None,
            comparison_settings: self.settings.clone(),
        };

        let comparison = DocumentComparison {
            id: uuid::Uuid::new_v4().to_string(),
            original_document_id: "original".to_string(),
            revised_document_id: "revised".to_string(),
            comparison_type: ComparisonType::WordLevel,
            changes,
            statistics,
            metadata,
            created_at: Utc::now(),
        };

        let elapsed = start_time.elapsed();
        info!("Document comparison completed in {:?}", elapsed);

        Ok(comparison)
    }

    fn preprocess_text(&self, text: &str) -> String {
        let mut processed = text.to_string();

        if self.settings.ignore_case {
            processed = processed.to_lowercase();
        }

        if self.settings.ignore_whitespace {
            processed = processed.split_whitespace().collect::<Vec<_>>().join(" ");
        }

        if self.settings.ignore_punctuation {
            processed = processed.chars()
                .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                .collect();
        }

        if self.settings.ignore_citations {
            processed = self.citation_regex.replace_all(&processed, "[CITATION]").to_string();
        }

        if self.settings.ignore_numbering {
            processed = self.numbering_regex.replace_all(&processed, "[NUMBER]").to_string();
        }

        processed
    }

    fn extract_changes(&self, diff: &TextDiff<'_, '_, str>, original: &str, revised: &str) -> Result<Vec<Change>> {
        let mut changes = Vec::new();
        let mut change_id = 0;

        let original_lines: Vec<&str> = original.lines().collect();
        let revised_lines: Vec<&str> = revised.lines().collect();

        let mut original_line_num = 0;
        let mut revised_line_num = 0;
        let mut original_offset = 0;
        let mut revised_offset = 0;

        for group in diff.grouped_ops(self.settings.context_lines as usize) {
            for op in group {
                match op.tag() {
                    ChangeTag::Equal => {
                        // Skip equal sections, just update positions
                        for i in op.old_range() {
                            if i < original_lines.len() {
                                original_offset += original_lines[i].len() + 1; // +1 for newline
                            }
                            original_line_num += 1;
                        }
                        for i in op.new_range() {
                            if i < revised_lines.len() {
                                revised_offset += revised_lines[i].len() + 1;
                            }
                            revised_line_num += 1;
                        }
                    }
                    ChangeTag::Delete => {
                        let deleted_text = op.old_range()
                            .filter_map(|i| original_lines.get(i))
                            .collect::<Vec<_>>()
                            .join("\n");

                        if deleted_text.len() >= self.settings.minimum_change_length as usize {
                            let change = Change {
                                id: format!("change_{}", change_id),
                                change_type: ChangeType::Delete,
                                original_text: Some(deleted_text.clone()),
                                revised_text: None,
                                position: TextPosition {
                                    start_line: original_line_num,
                                    start_column: 0,
                                    end_line: original_line_num + op.old_range().len() as u32,
                                    end_column: 0,
                                    start_offset: original_offset,
                                    end_offset: original_offset + deleted_text.len() as u32,
                                },
                                confidence: 1.0,
                                category: self.categorize_change(&deleted_text, None),
                                author: None,
                                timestamp: Utc::now(),
                                comment: None,
                                accepted: None,
                            };
                            changes.push(change);
                            change_id += 1;
                        }

                        for i in op.old_range() {
                            if i < original_lines.len() {
                                original_offset += original_lines[i].len() + 1;
                            }
                            original_line_num += 1;
                        }
                    }
                    ChangeTag::Insert => {
                        let inserted_text = op.new_range()
                            .filter_map(|i| revised_lines.get(i))
                            .collect::<Vec<_>>()
                            .join("\n");

                        if inserted_text.len() >= self.settings.minimum_change_length as usize {
                            let change = Change {
                                id: format!("change_{}", change_id),
                                change_type: ChangeType::Insert,
                                original_text: None,
                                revised_text: Some(inserted_text.clone()),
                                position: TextPosition {
                                    start_line: revised_line_num,
                                    start_column: 0,
                                    end_line: revised_line_num + op.new_range().len() as u32,
                                    end_column: 0,
                                    start_offset: revised_offset,
                                    end_offset: revised_offset + inserted_text.len() as u32,
                                },
                                confidence: 1.0,
                                category: self.categorize_change(None, &inserted_text),
                                author: None,
                                timestamp: Utc::now(),
                                comment: None,
                                accepted: None,
                            };
                            changes.push(change);
                            change_id += 1;
                        }

                        for i in op.new_range() {
                            if i < revised_lines.len() {
                                revised_offset += revised_lines[i].len() + 1;
                            }
                            revised_line_num += 1;
                        }
                    }
                    ChangeTag::Replace => {
                        let original_text = op.old_range()
                            .filter_map(|i| original_lines.get(i))
                            .collect::<Vec<_>>()
                            .join("\n");

                        let revised_text = op.new_range()
                            .filter_map(|i| revised_lines.get(i))
                            .collect::<Vec<_>>()
                            .join("\n");

                        if original_text.len() >= self.settings.minimum_change_length as usize ||
                           revised_text.len() >= self.settings.minimum_change_length as usize {
                            let change = Change {
                                id: format!("change_{}", change_id),
                                change_type: ChangeType::Replace,
                                original_text: Some(original_text.clone()),
                                revised_text: Some(revised_text.clone()),
                                position: TextPosition {
                                    start_line: original_line_num,
                                    start_column: 0,
                                    end_line: original_line_num + op.old_range().len() as u32,
                                    end_column: 0,
                                    start_offset: original_offset,
                                    end_offset: original_offset + original_text.len() as u32,
                                },
                                confidence: self.calculate_change_confidence(&original_text, &revised_text),
                                category: self.categorize_change(Some(&original_text), &revised_text),
                                author: None,
                                timestamp: Utc::now(),
                                comment: None,
                                accepted: None,
                            };
                            changes.push(change);
                            change_id += 1;
                        }

                        for i in op.old_range() {
                            if i < original_lines.len() {
                                original_offset += original_lines[i].len() + 1;
                            }
                            original_line_num += 1;
                        }

                        for i in op.new_range() {
                            if i < revised_lines.len() {
                                revised_offset += revised_lines[i].len() + 1;
                            }
                            revised_line_num += 1;
                        }
                    }
                }
            }
        }

        Ok(changes)
    }

    fn categorize_change(&self, original: Option<&str>, revised: &str) -> ChangeCategory {
        let text = revised;

        // Check for citations
        if self.citation_regex.is_match(text) {
            return ChangeCategory::Citation;
        }

        // Check for numbering
        if self.numbering_regex.is_match(text) {
            return ChangeCategory::Numbering;
        }

        // Check for formatting indicators
        if text.contains("**") || text.contains("__") || text.contains("*") {
            return ChangeCategory::Formatting;
        }

        // Check for substantive vs editorial changes
        let word_count = text.split_whitespace().count();
        if word_count > 5 {
            ChangeCategory::Substantive
        } else {
            ChangeCategory::Editorial
        }
    }

    fn calculate_change_confidence(&self, original: &str, revised: &str) -> f32 {
        let similarity = similar::TextDiff::from_words(original, revised)
            .ratio();
        
        // Confidence is inverse of similarity for changes
        1.0 - similarity as f32
    }

    fn calculate_statistics(&self, changes: &[Change], original: &str, revised: &str) -> ComparisonStatistics {
        let mut stats = ComparisonStatistics {
            total_changes: changes.len() as u32,
            insertions: 0,
            deletions: 0,
            replacements: 0,
            moves: 0,
            words_added: 0,
            words_removed: 0,
            characters_added: 0,
            characters_removed: 0,
            similarity_score: 0.0,
            change_density: 0.0,
        };

        for change in changes {
            match change.change_type {
                ChangeType::Insert => {
                    stats.insertions += 1;
                    if let Some(text) = &change.revised_text {
                        stats.words_added += text.split_whitespace().count() as u32;
                        stats.characters_added += text.len() as u32;
                    }
                }
                ChangeType::Delete => {
                    stats.deletions += 1;
                    if let Some(text) = &change.original_text {
                        stats.words_removed += text.split_whitespace().count() as u32;
                        stats.characters_removed += text.len() as u32;
                    }
                }
                ChangeType::Replace => {
                    stats.replacements += 1;
                    if let Some(original_text) = &change.original_text {
                        stats.words_removed += original_text.split_whitespace().count() as u32;
                        stats.characters_removed += original_text.len() as u32;
                    }
                    if let Some(revised_text) = &change.revised_text {
                        stats.words_added += revised_text.split_whitespace().count() as u32;
                        stats.characters_added += revised_text.len() as u32;
                    }
                }
                ChangeType::Move => {
                    stats.moves += 1;
                }
                _ => {}
            }
        }

        // Calculate similarity score
        let diff = similar::TextDiff::from_lines(original, revised);
        stats.similarity_score = diff.ratio() as f32;

        // Calculate change density (changes per 100 words)
        let total_words = original.split_whitespace().count() as f32;
        if total_words > 0.0 {
            stats.change_density = (stats.total_changes as f32 / total_words) * 100.0;
        }

        stats
    }

    pub fn generate_redline_html(&self, comparison: &DocumentComparison, original: &str, revised: &str) -> Result<RedlineDocument> {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<style>\n");
        html.push_str(".insertion { background-color: #d4edda; color: #155724; }\n");
        html.push_str(".deletion { background-color: #f8d7da; color: #721c24; text-decoration: line-through; }\n");
        html.push_str(".replacement-old { background-color: #f8d7da; color: #721c24; text-decoration: line-through; }\n");
        html.push_str(".replacement-new { background-color: #d4edda; color: #155724; }\n");
        html.push_str(".change-comment { font-style: italic; color: #6c757d; }\n");
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");

        // Apply changes to generate redlined version
        let mut result_text = revised.to_string();
        
        // Sort changes by position (reverse order to maintain positions)
        let mut sorted_changes = comparison.changes.clone();
        sorted_changes.sort_by(|a, b| b.position.start_offset.cmp(&a.position.start_offset));

        for change in &sorted_changes {
            match change.change_type {
                ChangeType::Insert => {
                    if let Some(text) = &change.revised_text {
                        let marked_text = format!("<span class=\"insertion\">{}</span>", html_escape::encode_text(text));
                        // Insert at position
                        result_text.insert_str(change.position.start_offset as usize, &marked_text);
                    }
                }
                ChangeType::Delete => {
                    if let Some(text) = &change.original_text {
                        let marked_text = format!("<span class=\"deletion\">{}</span>", html_escape::encode_text(text));
                        // Insert deleted text with strikethrough
                        result_text.insert_str(change.position.start_offset as usize, &marked_text);
                    }
                }
                ChangeType::Replace => {
                    if let (Some(original_text), Some(revised_text)) = (&change.original_text, &change.revised_text) {
                        let marked_text = format!(
                            "<span class=\"replacement-old\">{}</span><span class=\"replacement-new\">{}</span>",
                            html_escape::encode_text(original_text),
                            html_escape::encode_text(revised_text)
                        );
                        // Replace the text
                        let start = change.position.start_offset as usize;
                        let end = change.position.end_offset as usize;
                        if end <= result_text.len() {
                            result_text.replace_range(start..end, &marked_text);
                        }
                    }
                }
                _ => {}
            }
        }

        html.push_str(&result_text.replace('\n', "<br>\n"));
        html.push_str("\n</body>\n</html>");

        let mut metadata = HashMap::new();
        metadata.insert("total_changes".to_string(), comparison.statistics.total_changes.to_string());
        metadata.insert("similarity_score".to_string(), comparison.statistics.similarity_score.to_string());

        Ok(RedlineDocument {
            content: html,
            format: RedlineFormat::Html,
            changes: comparison.changes.clone(),
            metadata,
        })
    }

    pub fn accept_change(&self, comparison: &mut DocumentComparison, change_id: &str) -> Result<()> {
        if let Some(change) = comparison.changes.iter_mut().find(|c| c.id == change_id) {
            change.accepted = Some(true);
            info!("Accepted change: {}", change_id);
            Ok(())
        } else {
            Err(anyhow!("Change not found: {}", change_id))
        }
    }

    pub fn reject_change(&self, comparison: &mut DocumentComparison, change_id: &str) -> Result<()> {
        if let Some(change) = comparison.changes.iter_mut().find(|c| c.id == change_id) {
            change.accepted = Some(false);
            info!("Rejected change: {}", change_id);
            Ok(())
        } else {
            Err(anyhow!("Change not found: {}", change_id))
        }
    }

    pub fn add_comment(&self, comparison: &mut DocumentComparison, change_id: &str, comment: String) -> Result<()> {
        if let Some(change) = comparison.changes.iter_mut().find(|c| c.id == change_id) {
            change.comment = Some(comment);
            info!("Added comment to change: {}", change_id);
            Ok(())
        } else {
            Err(anyhow!("Change not found: {}", change_id))
        }
    }
}

impl Default for ComparisonSettings {
    fn default() -> Self {
        Self {
            ignore_whitespace: false,
            ignore_case: false,
            ignore_punctuation: false,
            ignore_formatting: false,
            ignore_citations: false,
            ignore_numbering: false,
            minimum_change_length: 1,
            context_lines: 3,
            word_wrap_threshold: 80,
        }
    }
}
