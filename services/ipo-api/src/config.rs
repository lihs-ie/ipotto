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
