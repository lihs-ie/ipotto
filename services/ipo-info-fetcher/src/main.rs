use std::env;
use std::net::SocketAddr;

use tracing_subscriber::EnvFilter;

mod application;
mod config;
mod domain;
mod error;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8082".to_string())
        .parse()
        .expect("PORT must be a valid u16");

    let router = presentation::routes::create_router();
    let address = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!("ipo-info-fetcher listening on {}", address);

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("failed to bind address");

    axum::serve(listener, router).await.expect("server error");
}
