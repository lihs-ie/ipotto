use reqwest::StatusCode;

/// Returns whether a status code is retryable.
pub fn is_retryable_status_code(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::REQUEST_TIMEOUT
            | StatusCode::TOO_MANY_REQUESTS
            | StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT
    )
}

#[cfg(test)]
mod tests {
    use super::is_retryable_status_code;
    use reqwest::StatusCode;

    #[test]
    fn transient_server_errors_are_retryable() {
        for status in [
            StatusCode::REQUEST_TIMEOUT,
            StatusCode::TOO_MANY_REQUESTS,
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::BAD_GATEWAY,
            StatusCode::SERVICE_UNAVAILABLE,
            StatusCode::GATEWAY_TIMEOUT,
        ] {
            assert!(
                is_retryable_status_code(status),
                "status {status} should be retryable"
            );
        }
    }

    #[test]
    fn client_errors_and_success_are_not_retryable() {
        for status in [
            StatusCode::OK,
            StatusCode::CREATED,
            StatusCode::BAD_REQUEST,
            StatusCode::UNAUTHORIZED,
            StatusCode::FORBIDDEN,
            StatusCode::NOT_FOUND,
            StatusCode::CONFLICT,
        ] {
            assert!(
                !is_retryable_status_code(status),
                "status {status} should not be retryable"
            );
        }
    }
}
