use ipo_backend_shared::http::HttpServiceConfig;

pub const HTTP_SERVICE_CONFIG: HttpServiceConfig =
    HttpServiceConfig::new("ipo-info-fetcher", 8082);
