use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

/// Two keywords extracted from image authentication mail content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageAuthenticationKeyword {
    first_keyword: String,
    second_keyword: String,
}

impl ImageAuthenticationKeyword {
    /// Creates image authentication keywords.
    pub fn new(
        first_keyword: impl Into<String>,
        second_keyword: impl Into<String>,
    ) -> Result<Self, DomainError> {
        let first_keyword = first_keyword.into().trim().to_string();
        let second_keyword = second_keyword.into().trim().to_string();
        if first_keyword.is_empty() || second_keyword.is_empty() {
            return Err(DomainError::MailParseError {
                reason: "image authentication keywords must not be empty".to_string(),
            });
        }
        Ok(Self {
            first_keyword,
            second_keyword,
        })
    }

    /// Returns the first keyword.
    pub fn first_keyword(&self) -> &str {
        &self.first_keyword
    }

    /// Returns the second keyword.
    pub fn second_keyword(&self) -> &str {
        &self.second_keyword
    }
}

#[cfg(test)]
mod tests {
    use super::ImageAuthenticationKeyword;

    #[test]
    fn creates_keyword_pair() {
        let keyword = ImageAuthenticationKeyword::new("apple", "banana").expect("keyword");
        assert_eq!(keyword.first_keyword(), "apple");
        assert_eq!(keyword.second_keyword(), "banana");
    }

    #[test]
    fn rejects_empty_keyword() {
        let result = ImageAuthenticationKeyword::new("", "banana");
        assert!(result.is_err());
    }
}
