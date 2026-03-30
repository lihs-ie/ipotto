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

    pub fn service_unavailable(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, code, message, None)
    }
}

impl From<DomainError> for ApiError {
    fn from(error: DomainError) -> Self {
        match error {
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
            | DomainError::InvalidSecuritiesCompany { .. } => Self::new(
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
