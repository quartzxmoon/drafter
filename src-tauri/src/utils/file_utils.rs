// File utilities for PA eDocket Desktop

use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Ensure a directory exists, creating it if necessary
pub async fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path).await?;
    }
    Ok(())
}

/// Get file size in bytes
pub async fn get_file_size(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path).await?;
    Ok(metadata.len())
}

/// Check if file exists
pub async fn file_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// Get file extension
pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

/// Generate unique filename to avoid conflicts
pub fn generate_unique_filename(base_path: &Path, filename: &str) -> PathBuf {
    let mut path = base_path.join(filename);
    let mut counter = 1;
    
    while path.exists() {
        if let Some(stem) = Path::new(filename).file_stem() {
            if let Some(ext) = Path::new(filename).extension() {
                let new_filename = format!(
                    "{}_{}.{}",
                    stem.to_string_lossy(),
                    counter,
                    ext.to_string_lossy()
                );
                path = base_path.join(new_filename);
            } else {
                let new_filename = format!("{}_{}", stem.to_string_lossy(), counter);
                path = base_path.join(new_filename);
            }
        }
        counter += 1;
    }
    
    path
}

/// Sanitize filename for safe filesystem usage
pub fn sanitize_filename(filename: &str) -> String {
    let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut sanitized = filename.to_string();
    
    for ch in invalid_chars {
        sanitized = sanitized.replace(ch, "_");
    }
    
    // Remove leading/trailing whitespace and dots
    sanitized = sanitized.trim().trim_matches('.').to_string();
    
    // Ensure filename is not empty
    if sanitized.is_empty() {
        sanitized = "document".to_string();
    }
    
    // Limit length
    if sanitized.len() > 255 {
        sanitized.truncate(255);
    }
    
    sanitized
}

/// Get MIME type from file extension
pub fn get_mime_type(path: &Path) -> &'static str {
    match get_file_extension(path).as_deref() {
        Some("pdf") => "application/pdf",
        Some("doc") => "application/msword",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("txt") => "text/plain",
        Some("html") => "text/html",
        Some("json") => "application/json",
        Some("csv") => "text/csv",
        Some("zip") => "application/zip",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        _ => "application/octet-stream",
    }
}

/// Copy file with progress callback
pub async fn copy_file_with_progress<F>(
    src: &Path,
    dst: &Path,
    mut progress_callback: F,
) -> Result<()>
where
    F: FnMut(u64, u64),
{
    let total_size = get_file_size(src).await?;
    let mut src_file = fs::File::open(src).await?;
    let mut dst_file = fs::File::create(dst).await?;
    
    let mut buffer = vec![0u8; 8192];
    let mut copied = 0u64;
    
    loop {
        let bytes_read = tokio::io::AsyncReadExt::read(&mut src_file, &mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        
        tokio::io::AsyncWriteExt::write_all(&mut dst_file, &buffer[..bytes_read]).await?;
        copied += bytes_read as u64;
        progress_callback(copied, total_size);
    }
    
    Ok(())
}

/// Clean up temporary files older than specified duration
pub async fn cleanup_temp_files(temp_dir: &Path, max_age_hours: u64) -> Result<()> {
    let cutoff_time = std::time::SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(max_age_hours * 3600))
        .unwrap_or(std::time::UNIX_EPOCH);
    
    let mut entries = fs::read_dir(temp_dir).await?;
    
    while let Some(entry) = entries.next_entry().await? {
        let metadata = entry.metadata().await?;
        
        if let Ok(modified) = metadata.modified() {
            if modified < cutoff_time {
                if metadata.is_file() {
                    let _ = fs::remove_file(entry.path()).await;
                } else if metadata.is_dir() {
                    let _ = fs::remove_dir_all(entry.path()).await;
                }
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("normal_file.txt"), "normal_file.txt");
        assert_eq!(sanitize_filename("file<with>bad:chars"), "file_with_bad_chars");
        assert_eq!(sanitize_filename("   .hidden   "), "hidden");
        assert_eq!(sanitize_filename(""), "document");
    }
    
    #[test]
    fn test_get_mime_type() {
        assert_eq!(get_mime_type(Path::new("test.pdf")), "application/pdf");
        assert_eq!(get_mime_type(Path::new("test.docx")), "application/vnd.openxmlformats-officedocument.wordprocessingml.document");
        assert_eq!(get_mime_type(Path::new("test.unknown")), "application/octet-stream");
    }
    
    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension(Path::new("test.PDF")), Some("pdf".to_string()));
        assert_eq!(get_file_extension(Path::new("test.TXT")), Some("txt".to_string()));
        assert_eq!(get_file_extension(Path::new("test")), None);
    }
    
    #[tokio::test]
    async fn test_ensure_dir_exists() {
        let temp_dir = tempdir().unwrap();
        let test_path = temp_dir.path().join("new_dir");
        
        assert!(!test_path.exists());
        ensure_dir_exists(&test_path).await.unwrap();
        assert!(test_path.exists());
    }
}
