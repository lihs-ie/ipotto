use std::time::Instant;

use axum::{extract::Request, middleware::Next, response::Response};
use ipo_backend_shared::logging::mask_email;
use tracing::{info, info_span, Instrument};
use ulid::Ulid;

use super::VerifiedToken;

/// Axum middleware that emits a structured span per incoming HTTP request.
///
/// Every request is wrapped in an `info_span!("http_request", ...)` that
/// records the generated `request_id`, HTTP method / path, and — when a
/// `VerifiedToken` has already been attached by `firebase_auth_middleware`
/// — the caller's UID and a masked email. The middleware logs two events:
/// `request_started` and `request_completed` (with status code + elapsed
/// milliseconds). Sensitive fields such as the raw `Authorization` header
/// and request / response bodies are intentionally never captured.
pub async fn request_logging_middleware(request: Request, next: Next) -> Response {
    let request_id = Ulid::new().to_string();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let (uid, masked_email) = extract_identity(&request);

    let span = info_span!(
        "http_request",
        request_id = %request_id,
        method = %method,
        path = %path,
        uid = uid.as_deref().unwrap_or(""),
        email = masked_email.as_deref().unwrap_or(""),
    );

    async move {
        info!(event = "request_started");
        let started_at = Instant::now();
        let response = next.run(request).await;
        info!(
            event = "request_completed",
            status = response.status().as_u16(),
            duration_ms = started_at.elapsed().as_millis() as u64,
        );
        response
    }
    .instrument(span)
    .await
}

fn extract_identity(request: &Request) -> (Option<String>, Option<String>) {
    match request.extensions().get::<VerifiedToken>() {
        Some(verified) => {
            let uid = Some(verified.uid.clone());
            let email = verified.email.as_deref().map(mask_email);
            (uid, email)
        }
        None => (None, None),
    }
}

#[cfg(test)]
mod tests {
    use super::{extract_identity, VerifiedToken};
    use axum::{body::Body, http::Request};

    #[test]
    fn extracts_masked_identity_when_verified_token_is_present() {
        let verified = VerifiedToken::new(
            "uid-123",
            Some("alice@example.com".to_string()),
            true,
            1_000,
            2_000,
        );
        let mut request = Request::builder()
            .uri("/api/v1/stocks")
            .body(Body::empty())
            .unwrap();
        request.extensions_mut().insert(verified);

        let (uid, email) = extract_identity(&request);
        assert_eq!(uid.as_deref(), Some("uid-123"));
        assert_eq!(email.as_deref(), Some("ali***@example.com"));
    }

    #[test]
    fn returns_none_when_verified_token_is_missing() {
        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let (uid, email) = extract_identity(&request);
        assert!(uid.is_none());
        assert!(email.is_none());
    }

    #[test]
    fn passes_through_verified_token_without_email() {
        let verified = VerifiedToken::new("uid-abc", None, false, 1_000, 2_000);
        let mut request = Request::builder()
            .uri("/api/v1/stocks")
            .body(Body::empty())
            .unwrap();
        request.extensions_mut().insert(verified);

        let (uid, email) = extract_identity(&request);
        assert_eq!(uid.as_deref(), Some("uid-abc"));
        assert!(email.is_none());
    }
}
