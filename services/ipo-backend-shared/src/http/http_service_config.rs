/// Shared HTTP service configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpServiceConfig {
    service_name: &'static str,
    default_port: u16,
}

impl HttpServiceConfig {
    /// Creates a new HTTP service configuration.
    pub const fn new(service_name: &'static str, default_port: u16) -> Self {
        Self {
            service_name,
            default_port,
        }
    }

    /// Returns the service name used for logs.
    pub const fn service_name(&self) -> &'static str {
        self.service_name
    }

    /// Returns the default port used when `PORT` is not set.
    pub const fn default_port(&self) -> u16 {
        self.default_port
    }
}

#[cfg(test)]
mod tests {
    use super::HttpServiceConfig;

    #[test]
    fn returns_config_values() {
        let config = HttpServiceConfig::new("svc", 8080);

        assert_eq!(config.service_name(), "svc");
        assert_eq!(config.default_port(), 8080);
    }
}
