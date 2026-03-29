use axum::Router;
use ipo_backend_shared::http::create_health_check_router;

/// Creates the result checker router.
pub fn create_router() -> Router {
    Router::new().merge(create_health_check_router())
}
