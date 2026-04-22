use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Serialize};

use crate::errors::DomainError;

use super::{is_retryable_status_code, retry_with_policy, RetryPolicy};

/// POSTs a JSON body to `url` and decodes the response body into `Res`,
/// wrapping the full round trip in a [`RetryPolicy`] driven
/// exponential-backoff loop.
///
/// Transport errors (connect / timeout), 5xx responses, 408 and 429 are
/// retried up to `policy.max_retries()` times. 4xx (apart from 408/429)
/// fail fast. JSON decode errors fail fast.
pub async fn post_json_with_retry<Req, Res>(
    policy: &RetryPolicy,
    client: &Client,
    url: &str,
    request: &Req,
) -> Result<Res, DomainError>
where
    Req: Serialize + ?Sized,
    Res: DeserializeOwned,
{
    let body = serde_json::to_vec(request).map_err(|error| DomainError::HttpClientError {
        reason: format!("failed to serialize request body: {error}"),
    })?;

    retry_with_policy(policy, RetryableHttpError::is_retryable, || {
        let body = body.clone();
        async move {
            let response = client
                .post(url)
                .header("content-type", "application/json")
                .body(body)
                .send()
                .await
                .map_err(RetryableHttpError::from_transport)?;
            decode_response(response).await
        }
    })
    .await
    .map_err(RetryableHttpError::into_domain)
}

/// GETs from `url` and decodes the response body into `Res`, wrapping
/// the full round trip in a [`RetryPolicy`] driven exponential-backoff
/// loop. Same retry semantics as [`post_json_with_retry`].
pub async fn get_json_with_retry<Res>(
    policy: &RetryPolicy,
    client: &Client,
    url: &str,
) -> Result<Res, DomainError>
where
    Res: DeserializeOwned,
{
    retry_with_policy(policy, RetryableHttpError::is_retryable, || async move {
        let response = client
            .get(url)
            .send()
            .await
            .map_err(RetryableHttpError::from_transport)?;
        decode_response(response).await
    })
    .await
    .map_err(RetryableHttpError::into_domain)
}

async fn decode_response<Res>(response: reqwest::Response) -> Result<Res, RetryableHttpError>
where
    Res: DeserializeOwned,
{
    let status = response.status();
    if !status.is_success() {
        return Err(classify_non_success_status(status));
    }
    response
        .json::<Res>()
        .await
        .map_err(|error| RetryableHttpError::Decode {
            reason: error.to_string(),
        })
}

fn classify_non_success_status(status: StatusCode) -> RetryableHttpError {
    let reason = format!("upstream returned HTTP {status}");
    if is_retryable_status_code(status) {
        RetryableHttpError::Status {
            reason,
            retryable: true,
        }
    } else {
        RetryableHttpError::Status {
            reason,
            retryable: false,
        }
    }
}

#[derive(Debug)]
enum RetryableHttpError {
    Transport { reason: String, retryable: bool },
    Status { reason: String, retryable: bool },
    Decode { reason: String },
}

impl RetryableHttpError {
    fn from_transport(error: reqwest::Error) -> Self {
        let retryable = error.is_connect() || error.is_timeout() || error.is_request();
        Self::Transport {
            reason: error.to_string(),
            retryable,
        }
    }

    fn is_retryable(&self) -> bool {
        match self {
            Self::Transport { retryable, .. } => *retryable,
            Self::Status { retryable, .. } => *retryable,
            Self::Decode { .. } => false,
        }
    }

    fn into_domain(self) -> DomainError {
        match self {
            Self::Transport { reason, .. } => DomainError::HttpClientError { reason },
            Self::Status { reason, .. } => DomainError::HttpClientError { reason },
            Self::Decode { reason } => DomainError::HttpClientError {
                reason: format!("failed to decode response body: {reason}"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Echo {
        id: u32,
    }

    fn fast_policy() -> RetryPolicy {
        RetryPolicy::new(3, 1, 2, 0.0)
    }

    #[tokio::test]
    async fn post_retries_on_503_then_succeeds() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/echo"))
            .respond_with(ResponseTemplate::new(503))
            .up_to_n_times(2)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/echo"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Echo { id: 7 }))
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let url = format!("{}/echo", server.uri());
        let request = Echo { id: 7 };
        let response: Echo = post_json_with_retry(&fast_policy(), &client, &url, &request)
            .await
            .expect("retry should succeed eventually");
        assert_eq!(response, Echo { id: 7 });
    }

    #[tokio::test]
    async fn post_does_not_retry_on_400() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/echo"))
            .respond_with(ResponseTemplate::new(400))
            .expect(1)
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let url = format!("{}/echo", server.uri());
        let request = Echo { id: 1 };
        let result: Result<Echo, _> =
            post_json_with_retry(&fast_policy(), &client, &url, &request).await;
        assert!(matches!(result, Err(DomainError::HttpClientError { .. })));
    }

    #[tokio::test]
    async fn get_retries_on_503_then_succeeds() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/echo"))
            .respond_with(ResponseTemplate::new(503))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/echo"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Echo { id: 3 }))
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let url = format!("{}/echo", server.uri());
        let response: Echo = get_json_with_retry(&fast_policy(), &client, &url)
            .await
            .expect("retry should succeed");
        assert_eq!(response, Echo { id: 3 });
    }

    #[tokio::test]
    async fn post_propagates_final_status_after_retry_budget() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/echo"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let url = format!("{}/echo", server.uri());
        let request = Echo { id: 1 };
        let result: Result<Echo, _> =
            post_json_with_retry(&fast_policy(), &client, &url, &request).await;
        assert!(matches!(result, Err(DomainError::HttpClientError { .. })));
    }
}
