use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiErrorDetailResponse {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiErrorResponse {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<ApiErrorDetailResponse>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ErrorEnvelopeResponse {
    pub error: ApiErrorResponse,
}
