use axum::http::StatusCode;

use crate::error::ApiError;

/// Classification of Firebase ID token verification failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirebaseAuthError {
    MissingAuthorizationHeader,
    InvalidBearerFormat,
    MalformedToken,
    UnsupportedAlgorithm,
    MissingKeyId,
    UnknownKeyId,
    InvalidSignature,
    ExpiredToken,
    InvalidIssuer,
    InvalidAudience,
    MissingSubject,
    ProviderNotSupported,
    JwksFetchFailed(String),
}

impl FirebaseAuthError {
    fn code(&self) -> &'static str {
        match self {
            Self::MissingAuthorizationHeader | Self::InvalidBearerFormat => {
                "AUTHORIZATION_HEADER_INVALID"
            }
            Self::MalformedToken
            | Self::UnsupportedAlgorithm
            | Self::MissingKeyId
            | Self::UnknownKeyId
            | Self::InvalidSignature => "TOKEN_INVALID",
            Self::ExpiredToken => "TOKEN_EXPIRED",
            Self::InvalidIssuer | Self::InvalidAudience | Self::MissingSubject => "TOKEN_REJECTED",
            Self::ProviderNotSupported => "AUTHORIZATION_PROVIDER_UNSUPPORTED",
            Self::JwksFetchFailed(_) => "AUTH_BACKEND_UNAVAILABLE",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::ProviderNotSupported => StatusCode::FORBIDDEN,
            Self::JwksFetchFailed(_) => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::UNAUTHORIZED,
        }
    }

    fn public_message(&self) -> &'static str {
        match self {
            Self::MissingAuthorizationHeader => "Authorization ヘッダが必要です",
            Self::InvalidBearerFormat => "Authorization ヘッダ形式が不正です",
            Self::MalformedToken => "IDトークンの形式が不正です",
            Self::UnsupportedAlgorithm => "IDトークンのアルゴリズムが許可されていません",
            Self::MissingKeyId => "IDトークンに kid が含まれていません",
            Self::UnknownKeyId => "IDトークンの署名鍵が見つかりません",
            Self::InvalidSignature => "IDトークンの署名検証に失敗しました",
            Self::ExpiredToken => "IDトークンが期限切れです",
            Self::InvalidIssuer => "IDトークンの発行者が不正です",
            Self::InvalidAudience => "IDトークンの audience が不正です",
            Self::MissingSubject => "IDトークンに subject が含まれていません",
            Self::ProviderNotSupported => "Google 認証以外のプロバイダはサポートされていません",
            Self::JwksFetchFailed(_) => "認証サービスとの通信に失敗しました",
        }
    }
}

impl std::fmt::Display for FirebaseAuthError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JwksFetchFailed(reason) => {
                write!(formatter, "JWKs fetch failed: {reason}")
            }
            other => write!(formatter, "{}", other.public_message()),
        }
    }
}

impl std::error::Error for FirebaseAuthError {}

impl From<FirebaseAuthError> for ApiError {
    fn from(error: FirebaseAuthError) -> Self {
        ApiError::new(error.status(), error.code(), error.public_message(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::FirebaseAuthError;
    use axum::http::StatusCode;

    #[test]
    fn maps_client_side_errors_to_401() {
        for error in [
            FirebaseAuthError::MissingAuthorizationHeader,
            FirebaseAuthError::InvalidBearerFormat,
            FirebaseAuthError::MalformedToken,
            FirebaseAuthError::UnsupportedAlgorithm,
            FirebaseAuthError::MissingKeyId,
            FirebaseAuthError::UnknownKeyId,
            FirebaseAuthError::InvalidSignature,
            FirebaseAuthError::ExpiredToken,
            FirebaseAuthError::InvalidIssuer,
            FirebaseAuthError::InvalidAudience,
            FirebaseAuthError::MissingSubject,
        ] {
            assert_eq!(
                error.status(),
                StatusCode::UNAUTHORIZED,
                "{error:?} should map to 401"
            );
        }
    }

    #[test]
    fn maps_backend_fetch_failure_to_503() {
        let error = FirebaseAuthError::JwksFetchFailed("connection refused".to_string());
        assert_eq!(error.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn maps_provider_not_supported_to_403() {
        let error = FirebaseAuthError::ProviderNotSupported;
        assert_eq!(error.status(), StatusCode::FORBIDDEN);
        assert_eq!(error.code(), "AUTHORIZATION_PROVIDER_UNSUPPORTED");
    }
}
