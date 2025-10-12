// Cryptographic utilities for PA eDocket Desktop

use anyhow::Result;
use sha2::{Digest, Sha256};

/// Calculate SHA-256 hash of data
pub fn calculate_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Calculate SHA-256 hash of a string
pub fn calculate_sha256_string(data: &str) -> String {
    calculate_sha256(data.as_bytes())
}

/// Calculate SHA-256 hash of a file
pub async fn calculate_file_hash(file_path: &str) -> Result<String> {
    let data = tokio::fs::read(file_path).await?;
    Ok(calculate_sha256(&data))
}

/// Generate a secure random string for IDs
pub fn generate_secure_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sha256_calculation() {
        let data = "Hello, World!";
        let hash = calculate_sha256_string(data);
        assert_eq!(hash, "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f");
    }
    
    #[test]
    fn test_secure_id_generation() {
        let id1 = generate_secure_id();
        let id2 = generate_secure_id();
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 36); // UUID format
    }
}
