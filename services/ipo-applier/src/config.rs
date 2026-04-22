use ipo_backend_shared::http::HttpServiceConfig;
use std::env;

pub const HTTP_SERVICE_CONFIG: HttpServiceConfig = HttpServiceConfig::new("ipo-applier", 8084);

pub fn ipo_browser_base_url() -> String {
    env::var("IPO_BROWSER_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string())
}
