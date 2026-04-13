use ipo_backend_shared::http::HttpServiceConfig;
use std::env;

pub const HTTP_SERVICE_CONFIG: HttpServiceConfig = HttpServiceConfig::new("ipo-api", 8080);

pub fn ipo_browser_base_url() -> String {
    env::var("IPO_BROWSER_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string())
}

pub fn notification_from_address() -> String {
    env::var("IPO_NOTIFICATION_FROM_ADDRESS").unwrap_or_else(|_| "no-reply@example.com".to_string())
}

pub fn sendgrid_api_key() -> Option<String> {
    env::var("SENDGRID_API_KEY")
        .ok()
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{ipo_browser_base_url, notification_from_address, sendgrid_api_key};

    #[test]
    fn reads_browser_url_from_environment_or_default() {
        unsafe {
            std::env::remove_var("IPO_BROWSER_BASE_URL");
        }
        assert_eq!(ipo_browser_base_url(), "http://127.0.0.1:3000");

        unsafe {
            std::env::set_var("IPO_BROWSER_BASE_URL", "http://127.0.0.1:9999");
        }
        assert_eq!(ipo_browser_base_url(), "http://127.0.0.1:9999");
        unsafe {
            std::env::remove_var("IPO_BROWSER_BASE_URL");
        }
    }

    #[test]
    fn reads_notification_sender_and_optional_sendgrid_key() {
        unsafe {
            std::env::remove_var("IPO_NOTIFICATION_FROM_ADDRESS");
            std::env::remove_var("SENDGRID_API_KEY");
        }
        assert_eq!(notification_from_address(), "no-reply@example.com");
        assert_eq!(sendgrid_api_key(), None);

        unsafe {
            std::env::set_var("IPO_NOTIFICATION_FROM_ADDRESS", "notify@example.com");
            std::env::set_var("SENDGRID_API_KEY", "sg-key");
        }
        assert_eq!(notification_from_address(), "notify@example.com");
        assert_eq!(sendgrid_api_key(), Some("sg-key".to_string()));
        unsafe {
            std::env::remove_var("IPO_NOTIFICATION_FROM_ADDRESS");
            std::env::remove_var("SENDGRID_API_KEY");
        }
    }
}
