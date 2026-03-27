use axum::Router;

use super::handlers;

pub fn create_router() -> Router {
    Router::new().merge(handlers::health::router())
}
