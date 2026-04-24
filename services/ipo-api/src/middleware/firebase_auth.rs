use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, HeaderMap},
    middleware::Next,
    response::Response,
};

use crate::error::ApiError;

use super::{FirebaseAuthError, FirebaseTokenVerifier};

/// Axum middleware that enforces a valid Firebase ID token on the wrapped
/// routes. Attach it via `Router::route_layer(from_fn_with_state(...))`
/// scoped to the authenticated subtree so `/health` and `/internal/pubsub/*`
/// remain unauthenticated.
pub async fn firebase_auth_middleware(
    State(verifier): State<Arc<FirebaseTokenVerifier>>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = extract_bearer_token(request.headers())?;
    let verified = verifier.verify(&token).await.map_err(ApiError::from)?;
    request.extensions_mut().insert(verified);
    Ok(next.run(request).await)
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<String, FirebaseAuthError> {
    let header = headers
        .get(AUTHORIZATION)
        .ok_or(FirebaseAuthError::MissingAuthorizationHeader)?;
    let value = header
        .to_str()
        .map_err(|_| FirebaseAuthError::InvalidBearerFormat)?;
    let trimmed = value.trim();
    let (scheme, token) = trimmed
        .split_once(' ')
        .ok_or(FirebaseAuthError::InvalidBearerFormat)?;
    if !scheme.eq_ignore_ascii_case("Bearer") {
        return Err(FirebaseAuthError::InvalidBearerFormat);
    }
    let token = token.trim();
    if token.is_empty() {
        return Err(FirebaseAuthError::InvalidBearerFormat);
    }
    Ok(token.to_string())
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue};

    use super::{extract_bearer_token, FirebaseAuthError};

    #[test]
    fn extracts_token_from_bearer_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer abc.def.ghi"),
        );
        let token = extract_bearer_token(&headers).expect("token extracted");
        assert_eq!(token, "abc.def.ghi");
    }

    #[test]
    fn treats_bearer_scheme_case_insensitively() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_static("bearer abc.def.ghi"),
        );
        let token = extract_bearer_token(&headers).expect("token extracted");
        assert_eq!(token, "abc.def.ghi");
    }

    #[test]
    fn rejects_missing_authorization_header() {
        let headers = HeaderMap::new();
        assert_eq!(
            extract_bearer_token(&headers).unwrap_err(),
            FirebaseAuthError::MissingAuthorizationHeader
        );
    }

    #[test]
    fn rejects_non_bearer_scheme() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("Basic abcdef"));
        assert_eq!(
            extract_bearer_token(&headers).unwrap_err(),
            FirebaseAuthError::InvalidBearerFormat
        );
    }

    #[test]
    fn rejects_empty_token_after_scheme() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("Bearer "));
        assert_eq!(
            extract_bearer_token(&headers).unwrap_err(),
            FirebaseAuthError::InvalidBearerFormat
        );
    }
}
