use axum::{routing::get, Json, Router};
use serde::Serialize;

/// Health check response payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct HealthCheckResponse {
    status: &'static str,
}

/// Creates a shared health check router.
pub fn create_health_check_router() -> Router {
    Router::new().route("/health", get(health_check))
}

async fn health_check() -> Json<HealthCheckResponse> {
    Json(HealthCheckResponse { status: "ok" })
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use tower::util::ServiceExt;

    use super::create_health_check_router;

    #[tokio::test]
    async fn returns_ok_health_response() {
        let response = create_health_check_router()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        assert_eq!(&body[..], br#"{"status":"ok"}"#);
    }
}
