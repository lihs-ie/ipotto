use chrono::NaiveDate;

use crate::errors::DomainError;

/// Parses `YYYY/MM/DD` or `YYYY-MM-DD` into a `NaiveDate`.
pub fn parse_japanese_date(value: &str) -> Result<NaiveDate, DomainError> {
    let normalized = value.trim().replace('/', "-");
    NaiveDate::parse_from_str(&normalized, "%Y-%m-%d").map_err(|error| DomainError::ScrapingError {
        scraper_source: "japanese_date_parser".to_string(),
        reason: error.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::parse_japanese_date;

    #[test]
    fn parses_supported_date_formats() {
        assert!(parse_japanese_date("2026/04/01").is_ok());
        assert!(parse_japanese_date("2026-04-01").is_ok());
    }
}
