//! Helpers for redacting sensitive values before logging or serializing
//! into API responses. All functions operate on `&str` and return owned
//! `String` so callers can embed the result in `tracing` fields, JSON
//! payloads, or error messages.

const KEEP_PREFIX_LEN: usize = 3;

/// Masks a value keeping the first [`KEEP_PREFIX_LEN`] characters, padding
/// the tail with `***`. Used for identifiers such as login IDs where
/// partial context is still useful for operators.
///
/// Values shorter than the configured prefix are collapsed to `***` so the
/// replacement never reveals the whole secret.
pub fn mask_prefix(value: &str) -> String {
    let trimmed = value.chars().count();
    if trimmed <= KEEP_PREFIX_LEN {
        return "***".to_string();
    }
    let prefix: String = value.chars().take(KEEP_PREFIX_LEN).collect();
    format!("{prefix}***")
}

/// Masks an email address by redacting the local part while keeping the
/// domain intact. Accepts malformed inputs gracefully by falling back to
/// [`mask_prefix`] when the `@` separator is missing.
pub fn mask_email(value: &str) -> String {
    let mut parts = value.split('@');
    let local = parts.next().unwrap_or_default();
    let domain = parts.next().unwrap_or_default();
    if domain.is_empty() {
        return mask_prefix(value);
    }
    format!("{}@{domain}", mask_prefix(local))
}

/// Returns a constant redaction string for values where no partial context
/// should ever be exposed (passwords, bearer tokens, IMAP secrets).
pub fn mask_secret() -> &'static str {
    "***"
}

#[cfg(test)]
mod tests {
    use super::{mask_email, mask_prefix, mask_secret};

    #[test]
    fn mask_prefix_keeps_first_three_characters() {
        assert_eq!(mask_prefix("alice_login"), "ali***");
    }

    #[test]
    fn mask_prefix_collapses_short_values() {
        assert_eq!(mask_prefix(""), "***");
        assert_eq!(mask_prefix("ab"), "***");
        assert_eq!(mask_prefix("abc"), "***");
    }

    #[test]
    fn mask_prefix_handles_multibyte_characters() {
        // 3 half-width chars' worth of multibyte input should still keep
        // only the first three code points.
        assert_eq!(mask_prefix("日本語テスト"), "日本語***");
    }

    #[test]
    fn mask_email_preserves_domain() {
        assert_eq!(mask_email("alice@example.com"), "ali***@example.com");
    }

    #[test]
    fn mask_email_falls_back_to_prefix_when_missing_at_sign() {
        assert_eq!(mask_email("ab"), "***");
        assert_eq!(mask_email("longer_value"), "lon***");
    }

    #[test]
    fn mask_email_masks_short_local_parts() {
        assert_eq!(mask_email("ab@example.com"), "***@example.com");
    }

    #[test]
    fn mask_secret_returns_fixed_redaction() {
        assert_eq!(mask_secret(), "***");
    }
}
