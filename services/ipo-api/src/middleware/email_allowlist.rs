use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::error::ApiError;

use super::VerifiedToken;

/// Allow-listed email addresses, normalized for case-insensitive matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailAllowlistConfig {
    allowed_emails: Vec<String>,
}

impl EmailAllowlistConfig {
    /// Constructs a new config, lowercasing and trimming every supplied
    /// address. Empty entries are dropped. Returns `Err` when the
    /// resulting list is empty so callers can fail closed at startup.
    pub fn new(entries: Vec<String>) -> Result<Self, String> {
        let allowed_emails: Vec<String> = entries
            .into_iter()
            .map(|entry| entry.trim().to_ascii_lowercase())
            .filter(|entry| !entry.is_empty())
            .collect();
        if allowed_emails.is_empty() {
            return Err("ALLOWED_EMAIL must contain at least one address".to_string());
        }
        Ok(Self { allowed_emails })
    }

    /// Loads the allow-list from the `ALLOWED_EMAIL` environment variable,
    /// which is parsed as a comma-separated list.
    pub fn from_env() -> Result<Self, String> {
        let raw =
            std::env::var("ALLOWED_EMAIL").map_err(|_| "ALLOWED_EMAIL is not set".to_string())?;
        let entries = raw.split(',').map(str::to_string).collect();
        Self::new(entries)
    }

    /// Returns whether the supplied email is in the allow-list. Comparison
    /// is case-insensitive and ignores surrounding whitespace.
    pub fn is_allowed(&self, email: &str) -> bool {
        let normalized = email.trim().to_ascii_lowercase();
        self.allowed_emails.iter().any(|entry| entry == &normalized)
    }

    #[cfg(test)]
    pub fn entries(&self) -> &[String] {
        &self.allowed_emails
    }
}

/// Axum middleware enforcing the email allow-list. Must be layered **inside**
/// [`firebase_auth_middleware`] so a `VerifiedToken` is already present in
/// the request extensions by the time this runs.
pub async fn email_allowlist_middleware(
    State(config): State<Arc<EmailAllowlistConfig>>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    check_allowlist(&config, request.extensions().get::<VerifiedToken>())?;
    Ok(next.run(request).await)
}

fn check_allowlist(
    config: &EmailAllowlistConfig,
    verified: Option<&VerifiedToken>,
) -> Result<(), ApiError> {
    let verified = verified
        .ok_or_else(|| ApiError::forbidden("EMAIL_REQUIRED", "認証情報が取得できませんでした"))?;
    let email = verified.email.as_deref().ok_or_else(|| {
        ApiError::forbidden(
            "EMAIL_MISSING",
            "認証情報にメールアドレスが含まれていません",
        )
    })?;
    if !config.is_allowed(email) {
        return Err(ApiError::forbidden(
            "EMAIL_NOT_ALLOWED",
            "アクセスが許可されていません",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{check_allowlist, EmailAllowlistConfig};
    use crate::middleware::VerifiedToken;
    use axum::{http::StatusCode, response::IntoResponse};

    fn verified_with_email(email: Option<&str>) -> VerifiedToken {
        VerifiedToken::new(
            "uid-abc",
            email.map(str::to_string),
            true,
            Some("google.com".to_string()),
            1_000,
            2_000,
        )
    }

    #[test]
    fn new_normalizes_case_and_whitespace() {
        let config =
            EmailAllowlistConfig::new(vec!["  Alice@Example.COM ".to_string()]).expect("config");
        assert_eq!(config.entries(), ["alice@example.com"]);
    }

    #[test]
    fn new_drops_empty_entries() {
        let config = EmailAllowlistConfig::new(vec![
            "".to_string(),
            "   ".to_string(),
            "bob@example.com".to_string(),
        ])
        .expect("config");
        assert_eq!(config.entries(), ["bob@example.com"]);
    }

    #[test]
    fn new_rejects_entirely_empty_list() {
        let result = EmailAllowlistConfig::new(vec!["".to_string(), "   ".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn from_env_parses_comma_separated_list() {
        // Scoped env-var mutation to avoid races with other tests; the env
        // var is restored before returning.
        let guard = EnvGuard::set("ALLOWED_EMAIL", "alice@example.com,Bob@example.com , ");
        let config = EmailAllowlistConfig::from_env().expect("config");
        assert_eq!(config.entries(), ["alice@example.com", "bob@example.com"]);
        drop(guard);
    }

    #[test]
    fn from_env_errors_when_variable_is_missing() {
        let guard = EnvGuard::remove("ALLOWED_EMAIL");
        assert!(EmailAllowlistConfig::from_env().is_err());
        drop(guard);
    }

    #[test]
    fn is_allowed_matches_case_insensitively_and_trims() {
        let config =
            EmailAllowlistConfig::new(vec!["alice@example.com".to_string()]).expect("config");
        assert!(config.is_allowed("ALICE@example.com"));
        assert!(config.is_allowed(" alice@example.com "));
        assert!(!config.is_allowed("mallory@example.com"));
    }

    #[test]
    fn check_allowlist_accepts_allowed_caller() {
        let config =
            EmailAllowlistConfig::new(vec!["alice@example.com".to_string()]).expect("config");
        let token = verified_with_email(Some("Alice@example.com"));
        let result = check_allowlist(&config, Some(&token));
        assert!(result.is_ok());
    }

    #[test]
    fn check_allowlist_returns_403_when_email_not_listed() {
        let config =
            EmailAllowlistConfig::new(vec!["alice@example.com".to_string()]).expect("config");
        let token = verified_with_email(Some("mallory@example.com"));
        let response = check_allowlist(&config, Some(&token))
            .unwrap_err()
            .into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn check_allowlist_returns_403_when_token_has_no_email() {
        let config =
            EmailAllowlistConfig::new(vec!["alice@example.com".to_string()]).expect("config");
        let token = verified_with_email(None);
        let response = check_allowlist(&config, Some(&token))
            .unwrap_err()
            .into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn check_allowlist_returns_403_when_verified_token_extension_is_missing() {
        let config =
            EmailAllowlistConfig::new(vec!["alice@example.com".to_string()]).expect("config");
        let response = check_allowlist(&config, None).unwrap_err().into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, previous }
        }

        fn remove(key: &'static str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }
}
