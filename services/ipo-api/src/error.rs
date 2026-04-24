use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use ipo_backend_shared::errors::DomainError;

use crate::presentation::dto::{ApiErrorDetailResponse, ApiErrorResponse, ErrorEnvelopeResponse};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    status: StatusCode,
    code: String,
    message: String,
    details: Option<Vec<ApiErrorDetailResponse>>,
}

impl ApiError {
    pub fn new(
        status: StatusCode,
        code: impl Into<String>,
        message: impl Into<String>,
        details: Option<Vec<ApiErrorDetailResponse>>,
    ) -> Self {
        Self {
            status,
            code: code.into(),
            message: message.into(),
            details,
        }
    }

    pub fn validation(message: impl Into<String>, field: impl Into<String>) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            message,
            Some(vec![ApiErrorDetailResponse {
                field: field.into(),
                message: "入力値を確認してください".to_string(),
            }]),
        )
    }

    pub fn conflict(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, code, message, None)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "NOT_FOUND", message, None)
    }

    pub fn forbidden(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, code, message, None)
    }

    pub fn service_unavailable(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, code, message, None)
    }
}

impl From<DomainError> for ApiError {
    fn from(error: DomainError) -> Self {
        match error {
            DomainError::ValidationError { field, message } => Self::validation(message, field),
            DomainError::NotFound {
                resource,
                identifier,
            } => Self::not_found(format!("{resource} not found: {identifier}")),
            DomainError::DuplicateExclusion { .. } => {
                Self::conflict("CONFLICT", "同一企業名の除外銘柄が既に存在します")
            }
            DomainError::InvalidCompanyName { .. }
            | DomainError::InvalidExclusionReason { .. }
            | DomainError::InvalidMailAddress { .. }
            | DomainError::InvalidImapHost { .. }
            | DomainError::InvalidImapPort { .. }
            | DomainError::IncompleteCredential
            | DomainError::InvalidChannelDestination { .. }
            | DomainError::DuplicateChannelType { .. }
            | DomainError::NoActiveChannel
            | DomainError::InvalidMarket { .. }
            | DomainError::InvalidTickerSymbol { .. }
            | DomainError::InvalidIndustry { .. }
            | DomainError::InvalidPriceRange { .. }
            | DomainError::InvalidShares { .. }
            | DomainError::InvalidYen { .. }
            | DomainError::InvalidSchedule { .. }
            | DomainError::InvalidSecuritiesCompany { .. }
            | DomainError::InvalidStatusTransition { .. }
            | DomainError::OperationLogValidationError { .. } => Self::new(
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                error.to_string(),
                None,
            ),
            DomainError::NotificationSendError { .. }
            | DomainError::PubSubPublishError { .. }
            | DomainError::HttpClientError { .. }
            | DomainError::GmailApiError { .. }
            | DomainError::ImapError { .. }
            | DomainError::MailRetrievalTimeout => Self::service_unavailable(
                "SERVICE_UNAVAILABLE",
                "外部サービスとの通信に失敗しました",
            ),
            other => Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                other.to_string(),
                None,
            ),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status;
        let body = ErrorEnvelopeResponse {
            error: ApiErrorResponse {
                code: self.code,
                message: self.message,
                details: self.details,
            },
        };
        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::{http::StatusCode, response::IntoResponse};
    use ipo_backend_shared::errors::DomainError;

    use super::ApiError;

    #[test]
    fn maps_validation_errors_to_bad_request() {
        let response = ApiError::from(DomainError::ValidationError {
            field: "status".to_string(),
            message: "invalid status value: nope".to_string(),
        })
        .into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn maps_not_found_errors_to_not_found() {
        let response = ApiError::from(DomainError::NotFound {
            resource: "stock".to_string(),
            identifier: "01H...".to_string(),
        })
        .into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn forbidden_returns_403_with_supplied_code_and_message() {
        let response = ApiError::forbidden("EMAIL_NOT_ALLOWED", "メールが許可リストにありません")
            .into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn maps_operation_log_validation_errors_to_bad_request() {
        let response = ApiError::from(DomainError::OperationLogValidationError {
            reason: "invalid date".to_string(),
        })
        .into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
