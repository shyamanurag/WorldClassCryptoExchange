pub mod logging;
pub mod metrics;

use chrono::{DateTime, NaiveDateTime, Utc};
use uuid::Uuid;

/// Generates a unique ID for entities
pub fn generate_id(prefix: &str) -> String {
    format!("{}_{}", prefix, Uuid::new_v4())
}

/// Converts a timestamp in milliseconds to a DateTime<Utc>
pub fn timestamp_to_datetime(timestamp_ms: u64) -> DateTime<Utc> {
    let naive = NaiveDateTime::from_timestamp_millis(timestamp_ms as i64)
        .unwrap_or_else(|| {
            // Fallback to current time if timestamp is invalid
            let now = Utc::now();
            now.naive_utc()
        });
    
    DateTime::from_naive_utc_and_offset(naive, Utc)
}

/// Converts a DateTime<Utc> to a timestamp in milliseconds
pub fn datetime_to_timestamp(dt: &DateTime<Utc>) -> u64 {
    dt.timestamp_millis() as u64
}

/// Formats a price for display with appropriate decimal places
pub fn format_price(price: f64, decimals: usize) -> String {
    format!("{:.*}", decimals, price)
}

/// Formats a quantity for display with appropriate decimal places
pub fn format_quantity(quantity: f64, decimals: usize) -> String {
    format!("{:.*}", decimals, quantity)
}

/// Calculates the fee for a trade based on the price, quantity, and fee rate
pub fn calculate_fee(price: f64, quantity: f64, fee_rate: f64) -> f64 {
    price * quantity * fee_rate
}

/// Returns a standard error message for database errors
pub fn db_error_message(error: &str) -> String {
    format!("Database error: {}", error)
}

/// Returns a standard error message for API errors
pub fn api_error_message(error: &str) -> String {
    format!("API error: {}", error)
}

/// Returns a standard error message for validation errors
pub fn validation_error_message(error: &str) -> String {
    format!("Validation error: {}", error)
}

/// Sanitizes a string for use in logs (removes sensitive information)
pub fn sanitize_for_log(input: &str) -> String {
    // Remove potential sensitive information
    let mut output = input.to_string();
    
    // Remove potential JWT tokens
    if output.contains("Bearer ") {
        output = output.replace(
            &output.split("Bearer ")
                .nth(1)
                .unwrap_or("")
                .split('"')
                .next()
                .unwrap_or(""),
            "[REDACTED]"
        );
    }
    
    // Remove potential API keys
    if output.contains("api-key") || output.contains("apikey") {
        output = output.replace(
            &output.split("api-key:")
                .nth(1)
                .unwrap_or("")
                .split('"')
                .next()
                .unwrap_or(""),
            "[REDACTED]"
        );
    }
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_id() {
        let id = generate_id("test");
        assert!(id.starts_with("test_"));
        assert_eq!(id.len(), 41); // "test_" + 36 chars for UUID
    }
    
    #[test]
    fn test_timestamp_to_datetime() {
        let timestamp = 1620000000000; // May 3, 2021 12:26:40 AM GMT
        let dt = timestamp_to_datetime(timestamp);
        assert_eq!(dt.timestamp_millis(), timestamp as i64);
    }
    
    #[test]
    fn test_datetime_to_timestamp() {
        let now = Utc::now();
        let timestamp = datetime_to_timestamp(&now);
        let dt = timestamp_to_datetime(timestamp);
        
        // Allow for small differences due to precision loss
        let diff = (now.timestamp_millis() - dt.timestamp_millis()).abs();
        assert!(diff < 10);
    }
    
    #[test]
    fn test_format_price() {
        assert_eq!(format_price(123.456789, 2), "123.46");
        assert_eq!(format_price(123.456789, 4), "123.4568");
        assert_eq!(format_price(123.0, 2), "123.00");
    }
    
    #[test]
    fn test_format_quantity() {
        assert_eq!(format_quantity(123.456789, 2), "123.46");
        assert_eq!(format_quantity(123.456789, 4), "123.4568");
        assert_eq!(format_quantity(123.0, 2), "123.00");
    }
    
    #[test]
    fn test_calculate_fee() {
        assert_eq!(calculate_fee(100.0, 2.0, 0.001), 0.2);
        assert_eq!(calculate_fee(100.0, 2.0, 0.0025), 0.5);
    }
    
    #[test]
    fn test_sanitize_for_log() {
        let input = r#"{"authorization": "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"}"#;
        let sanitized = sanitize_for_log(input);
        assert!(sanitized.contains("[REDACTED]"));
        assert!(!sanitized.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
    }
}
