use ipo_backend_shared::http::run_http_service;

mod application;
mod config;
mod domain;
mod error;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() {
    let container = infrastructure::DependencyContainer::new()
        .expect("failed to construct ipo-api dependency container");
    let router = presentation::routes::create_router(container);
    run_http_service(config::HTTP_SERVICE_CONFIG, router)
        .await
        .unwrap_or_else(|error| panic!("failed to run http service: {error}"));
}
