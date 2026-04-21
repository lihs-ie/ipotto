use std::time::Duration;

use reqwest::Client;

use crate::errors::DomainError;

use super::HttpClientConfig;

/// Factory for reqwest clients with shared defaults.
#[derive(Debug, Clone)]
pub struct ReqwestClientFactory {
    config: HttpClientConfig,
}

impl ReqwestClientFactory {
    /// Creates a new reqwest client factory.
    pub fn new(config: HttpClientConfig) -> Self {
        Self { config }
    }

    /// Builds a reqwest client.
    pub fn build(&self) -> Result<Client, DomainError> {
        let builder = Client::builder()
            .connect_timeout(self.config.connect_timeout())
            .timeout(self.config.read_timeout())
            .pool_max_idle_per_host(self.config.max_idle_per_host());

        let builder = if self.config.keep_alive() {
            builder.pool_idle_timeout(Some(Duration::from_secs(90)))
        } else {
            builder.pool_idle_timeout(Some(Duration::ZERO))
        };

        builder
            .build()
            .map_err(|error| DomainError::HttpClientError {
                reason: error.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{HttpClientConfig, ReqwestClientFactory};

    #[test]
    fn builds_client_with_keep_alive_enabled() {
        let config = HttpClientConfig::new(5_000, 15_000, 8, true);
        let factory = ReqwestClientFactory::new(config);
        let result = factory.build();
        assert!(result.is_ok(), "expected client build to succeed");
    }

    #[test]
    fn builds_client_with_keep_alive_disabled() {
        let config = HttpClientConfig::new(1_000, 2_000, 1, false);
        let factory = ReqwestClientFactory::new(config);
        let result = factory.build();
        assert!(result.is_ok(), "expected client build to succeed");
    }
}
