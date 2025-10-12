// Validation utilities for PA eDocket Desktop

use regex::Regex;
use std::sync::OnceLock;

// Regex patterns for common validations
static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();
static PHONE_REGEX: OnceLock<Regex> = OnceLock::new();
static DOCKET_REGEX: OnceLock<Regex> = OnceLock::new();
static OTN_REGEX: OnceLock<Regex> = OnceLock::new();
static SID_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_email_regex() -> &'static Regex {
    EMAIL_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
    })
}

fn get_phone_regex() -> &'static Regex {
    PHONE_REGEX.get_or_init(|| {
        Regex::new(r"^\+?1?[-.\s]?\(?([0-9]{3})\)?[-.\s]?([0-9]{3})[-.\s]?([0-9]{4})$").unwrap()
    })
}

fn get_docket_regex() -> &'static Regex {
    DOCKET_REGEX.get_or_init(|| {
        // PA docket format: CP-##-CR-#######-#### or similar
        Regex::new(r"^[A-Z]{2}-\d{2}-[A-Z]{2}-\d{7}-\d{4}$").unwrap()
    })
}

fn get_otn_regex() -> &'static Regex {
    OTN_REGEX.get_or_init(|| {
        // PA OTN format: A ########-#
        Regex::new(r"^[A-Z]\s\d{8}-\d$").unwrap()
    })
}

fn get_sid_regex() -> &'static Regex {
    SID_REGEX.get_or_init(|| {
        // PA SID format: A#######
        Regex::new(r"^[A-Z]\d{7}$").unwrap()
    })
}

/// Validate email address
pub fn is_valid_email(email: &str) -> bool {
    get_email_regex().is_match(email)
}

/// Validate phone number
pub fn is_valid_phone(phone: &str) -> bool {
    get_phone_regex().is_match(phone)
}

/// Validate PA docket number
pub fn is_valid_docket_number(docket: &str) -> bool {
    get_docket_regex().is_match(docket)
}

/// Validate PA OTN (Originating Tracking Number)
pub fn is_valid_otn(otn: &str) -> bool {
    get_otn_regex().is_match(otn)
}

/// Validate PA SID (State ID Number)
pub fn is_valid_sid(sid: &str) -> bool {
    get_sid_regex().is_match(sid)
}

/// Normalize phone number to standard format
pub fn normalize_phone(phone: &str) -> Option<String> {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    
    if digits.len() == 10 {
        Some(format!("({}) {}-{}", &digits[0..3], &digits[3..6], &digits[6..10]))
    } else if digits.len() == 11 && digits.starts_with('1') {
        Some(format!("({}) {}-{}", &digits[1..4], &digits[4..7], &digits[7..11]))
    } else {
        None
    }
}

/// Normalize docket number to standard format
pub fn normalize_docket_number(docket: &str) -> String {
    docket.to_uppercase().replace(" ", "").replace("-", "-")
}

/// Validate and normalize OTN
pub fn normalize_otn(otn: &str) -> Option<String> {
    let cleaned = otn.to_uppercase().replace(" ", " ");
    if is_valid_otn(&cleaned) {
        Some(cleaned)
    } else {
        None
    }
}

/// Validate and normalize SID
pub fn normalize_sid(sid: &str) -> Option<String> {
    let cleaned = sid.to_uppercase().replace(" ", "");
    if is_valid_sid(&cleaned) {
        Some(cleaned)
    } else {
        None
    }
}

/// Validate required string field
pub fn validate_required_string(value: &str, field_name: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{} is required", field_name))
    } else {
        Ok(())
    }
}

/// Validate string length
pub fn validate_string_length(
    value: &str,
    field_name: &str,
    min_length: Option<usize>,
    max_length: Option<usize>,
) -> Result<(), String> {
    let len = value.len();
    
    if let Some(min) = min_length {
        if len < min {
            return Err(format!("{} must be at least {} characters", field_name, min));
        }
    }
    
    if let Some(max) = max_length {
        if len > max {
            return Err(format!("{} must be no more than {} characters", field_name, max));
        }
    }
    
    Ok(())
}

/// Validate numeric range
pub fn validate_numeric_range<T>(
    value: T,
    field_name: &str,
    min_value: Option<T>,
    max_value: Option<T>,
) -> Result<(), String>
where
    T: PartialOrd + std::fmt::Display,
{
    if let Some(min) = min_value {
        if value < min {
            return Err(format!("{} must be at least {}", field_name, min));
        }
    }
    
    if let Some(max) = max_value {
        if value > max {
            return Err(format!("{} must be no more than {}", field_name, max));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_email_validation() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name+tag@domain.co.uk"));
        assert!(!is_valid_email("invalid.email"));
        assert!(!is_valid_email("@domain.com"));
        assert!(!is_valid_email("user@"));
    }
    
    #[test]
    fn test_phone_validation() {
        assert!(is_valid_phone("(555) 123-4567"));
        assert!(is_valid_phone("555-123-4567"));
        assert!(is_valid_phone("5551234567"));
        assert!(is_valid_phone("+1 555 123 4567"));
        assert!(!is_valid_phone("123-456"));
        assert!(!is_valid_phone("abc-def-ghij"));
    }
    
    #[test]
    fn test_docket_validation() {
        assert!(is_valid_docket_number("CP-51-CR-1234567-2024"));
        assert!(is_valid_docket_number("MD-12-CV-0001234-2023"));
        assert!(!is_valid_docket_number("invalid-docket"));
        assert!(!is_valid_docket_number("CP-51-CR-123-24"));
    }
    
    #[test]
    fn test_phone_normalization() {
        assert_eq!(normalize_phone("5551234567"), Some("(555) 123-4567".to_string()));
        assert_eq!(normalize_phone("15551234567"), Some("(555) 123-4567".to_string()));
        assert_eq!(normalize_phone("(555) 123-4567"), Some("(555) 123-4567".to_string()));
        assert_eq!(normalize_phone("123456"), None);
    }
    
    #[test]
    fn test_string_validation() {
        assert!(validate_required_string("test", "field").is_ok());
        assert!(validate_required_string("", "field").is_err());
        assert!(validate_required_string("   ", "field").is_err());
        
        assert!(validate_string_length("test", "field", Some(1), Some(10)).is_ok());
        assert!(validate_string_length("", "field", Some(1), None).is_err());
        assert!(validate_string_length("very long string", "field", None, Some(5)).is_err());
    }
}
