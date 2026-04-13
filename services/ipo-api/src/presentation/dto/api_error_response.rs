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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{ApiErrorDetailResponse, ApiErrorResponse, ErrorEnvelopeResponse};

    #[test]
    fn serializes_error_responses() {
        let response = ErrorEnvelopeResponse {
            error: ApiErrorResponse {
                code: "VALIDATION_ERROR".to_string(),
                message: "invalid request".to_string(),
                details: Some(vec![ApiErrorDetailResponse {
                    field: "targetDate".to_string(),
                    message: "入力値を確認してください".to_string(),
                }]),
            },
        };

        assert_eq!(
            serde_json::to_value(response).expect("json"),
            json!({
                "error": {
                    "code": "VALIDATION_ERROR",
                    "message": "invalid request",
                    "details": [
                        {
                            "field": "targetDate",
                            "message": "入力値を確認してください"
                        }
                    ]
                }
            })
        );
    }
}
