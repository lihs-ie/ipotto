use crate::errors::DomainError;

/// Parses a yen string into an integer amount.
pub fn parse_japanese_yen(value: &str) -> Result<i64, DomainError> {
    let sanitized = value.trim().replace(['円', ',', '¥'], "");
    sanitized
        .parse::<i64>()
        .map_err(|error| DomainError::ScrapingError {
            scraper_source: "japanese_yen_parser".to_string(),
            reason: error.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::parse_japanese_yen;

    #[test]
    fn parses_yen_strings() {
        assert_eq!(parse_japanese_yen("1,500円").expect("yen"), 1500);
    }
}
