use regex::Regex;

use crate::{domain::account::ImageAuthenticationKeyword, errors::DomainError};

/// Parses image authentication keywords from mail body text.
pub fn parse_image_authentication_keyword(
    body: &str,
) -> Result<ImageAuthenticationKeyword, DomainError> {
    let pattern =
        Regex::new(r"([^\s+]+)\s*\+\s*([^\s+]+)").map_err(|error| DomainError::MailParseError {
            reason: error.to_string(),
        })?;
    let captures = pattern
        .captures(body)
        .ok_or_else(|| DomainError::MailParseError {
            reason: "could not extract two keywords separated by '+'".to_string(),
        })?;
    let first = captures
        .get(1)
        .ok_or_else(|| DomainError::MailParseError {
            reason: "missing first keyword".to_string(),
        })?
        .as_str();
    let second = captures
        .get(2)
        .ok_or_else(|| DomainError::MailParseError {
            reason: "missing second keyword".to_string(),
        })?
        .as_str();
    ImageAuthenticationKeyword::new(first, second)
}

#[cfg(test)]
mod tests {
    use super::parse_image_authentication_keyword;

    #[test]
    fn parses_keyword_pair() {
        let keyword = parse_image_authentication_keyword("みかん + りんご").expect("keyword");
        assert_eq!(keyword.first_keyword(), "みかん");
        assert_eq!(keyword.second_keyword(), "りんご");
    }

    #[test]
    fn rejects_missing_separator() {
        assert!(parse_image_authentication_keyword("みかん りんご").is_err());
    }
}
