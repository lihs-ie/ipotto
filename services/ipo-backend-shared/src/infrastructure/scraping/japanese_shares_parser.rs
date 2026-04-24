use crate::errors::DomainError;

/// Parses a shares string into a quantity.
pub fn parse_japanese_shares(value: &str) -> Result<u32, DomainError> {
    let sanitized = value.trim().replace("株", "").replace(',', "");
    sanitized
        .parse::<u32>()
        .map_err(|error| DomainError::ScrapingError {
            scraper_source: "japanese_shares_parser".to_string(),
            reason: error.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::parse_japanese_shares;

    #[test]
    fn parses_share_strings() {
        assert_eq!(parse_japanese_shares("10,000株").expect("shares"), 10_000);
    }
}
