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
            builder.pool_idle_timeout(Duration::from_secs(0))
        };

        builder
            .build()
            .map_err(|error| DomainError::NotificationSendError {
                channel_type: "http_client".to_string(),
                reason: error.to_string(),
            })
    }
}
