// Date utilities for PA eDocket Desktop

use chrono::{DateTime, NaiveDate, Utc};
use anyhow::Result;

/// Parse a date string in various common formats
pub fn parse_date_flexible(date_str: &str) -> Result<DateTime<Utc>> {
    let formats = [
        "%Y-%m-%d",
        "%m/%d/%Y",
        "%m-%d-%Y",
        "%m/%d/%y",
        "%B %d, %Y",
        "%b %d, %Y",
        "%d %B %Y",
        "%d %b %Y",
    ];
    
    for format in &formats {
        if let Ok(naive_date) = NaiveDate::parse_from_str(date_str, format) {
            return Ok(naive_date.and_hms_opt(0, 0, 0).unwrap().and_utc());
        }
    }
    
    Err(anyhow::anyhow!("Could not parse date: {}", date_str))
}

/// Format a date for display
pub fn format_date_display(date: &DateTime<Utc>) -> String {
    date.format("%B %d, %Y").to_string()
}

/// Format a date for API calls
pub fn format_date_api(date: &DateTime<Utc>) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Get current timestamp as ISO string
pub fn current_timestamp() -> String {
    Utc::now().to_rfc3339()
}

/// Check if a date is within a range
pub fn is_date_in_range(
    date: &DateTime<Utc>,
    start: Option<&DateTime<Utc>>,
    end: Option<&DateTime<Utc>>,
) -> bool {
    if let Some(start) = start {
        if date < start {
            return false;
        }
    }
    
    if let Some(end) = end {
        if date > end {
            return false;
        }
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    
    #[test]
    fn test_parse_date_flexible() {
        let test_cases = [
            "2024-01-15",
            "01/15/2024",
            "01-15-2024",
            "01/15/24",
            "January 15, 2024",
            "Jan 15, 2024",
        ];
        
        for date_str in &test_cases {
            let result = parse_date_flexible(date_str);
            assert!(result.is_ok(), "Failed to parse: {}", date_str);
        }
    }
    
    #[test]
    fn test_date_formatting() {
        let date = Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap();
        
        assert_eq!(format_date_display(&date), "January 15, 2024");
        assert_eq!(format_date_api(&date), "2024-01-15");
    }
    
    #[test]
    fn test_date_range_check() {
        let date = Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap();
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap();
        
        assert!(is_date_in_range(&date, Some(&start), Some(&end)));
        assert!(is_date_in_range(&date, None, Some(&end)));
        assert!(is_date_in_range(&date, Some(&start), None));
        assert!(is_date_in_range(&date, None, None));
    }
}
