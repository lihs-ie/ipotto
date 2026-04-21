/// Claims extracted from a successfully verified Firebase ID token.
///
/// Attached to the request via `request.extensions_mut().insert(...)` by
/// [`firebase_auth_middleware`] so downstream handlers and authorization
/// middlewares can access the caller's identity without re-parsing the JWT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedToken {
    pub uid: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub issued_at: i64,
    pub expires_at: i64,
}

impl VerifiedToken {
    pub fn new(
        uid: impl Into<String>,
        email: Option<String>,
        email_verified: bool,
        issued_at: i64,
        expires_at: i64,
    ) -> Self {
        Self {
            uid: uid.into(),
            email,
            email_verified,
            issued_at,
            expires_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VerifiedToken;

    #[test]
    fn preserves_supplied_claims() {
        let token = VerifiedToken::new(
            "uid-123",
            Some("user@example.com".to_string()),
            true,
            1_000,
            2_000,
        );
        assert_eq!(token.uid, "uid-123");
        assert_eq!(token.email.as_deref(), Some("user@example.com"));
        assert!(token.email_verified);
        assert_eq!(token.issued_at, 1_000);
        assert_eq!(token.expires_at, 2_000);
    }
}
