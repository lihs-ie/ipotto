use std::{env, error::Error, io, net::SocketAddr, num::ParseIntError};

use axum::Router;
use tracing_subscriber::{fmt::SubscriberBuilder, EnvFilter};

use super::http_service_config::HttpServiceConfig;

/// Runs a shared Axum HTTP service.
pub async fn run_http_service(
    config: HttpServiceConfig,
    router: Router,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    initialize_json_tracing()?;
    let port = read_port_from_env(config.default_port())?;
    serve_router(config.service_name(), port, router).await?;
    Ok(())
}

fn initialize_json_tracing() -> Result<(), Box<dyn Error + Send + Sync>> {
    SubscriberBuilder::default()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .try_init()
}

fn read_port_from_env(default_port: u16) -> Result<u16, ParseIntError> {
    resolve_port(env::var("PORT").ok().as_deref(), default_port)
}

async fn serve_router(service_name: &str, port: u16, router: Router) -> io::Result<()> {
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("{service_name} listening on {address}");

    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, router).await
}

fn resolve_port(port_env: Option<&str>, default_port: u16) -> Result<u16, ParseIntError> {
    match port_env {
        Some(value) => value.parse(),
        None => Ok(default_port),
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_port;

    #[test]
    fn resolves_env_port_when_present() {
        let port = resolve_port(Some("8080"), 3000).expect("port");
        assert_eq!(port, 8080);
    }

    #[test]
    fn resolves_default_port_when_env_missing() {
        let port = resolve_port(None, 3000).expect("port");
        assert_eq!(port, 3000);
    }

    #[test]
    fn rejects_invalid_port() {
        assert!(resolve_port(Some("invalid"), 3000).is_err());
    }
}
