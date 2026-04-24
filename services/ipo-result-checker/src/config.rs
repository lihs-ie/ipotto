use ipo_backend_shared::http::HttpServiceConfig;
use std::env;

pub const HTTP_SERVICE_CONFIG: HttpServiceConfig =
    HttpServiceConfig::new("ipo-result-checker", 8083);

pub fn ipo_browser_base_url() -> String {
    env::var("IPO_BROWSER_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string())
}

pub fn firebase_project_id() -> String {
    env::var("FIREBASE_PROJECT_ID").unwrap_or_else(|_| "ipotto-local".to_string())
}

pub fn credential_kek_name() -> String {
    env::var("IPOTTO_CREDENTIAL_KEK_NAME").unwrap_or_default()
}
