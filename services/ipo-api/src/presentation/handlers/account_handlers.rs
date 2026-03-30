use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    application::use_cases::{
        DeleteSecuritiesAccountUseCase, ListSecuritiesAccountsOutput,
        ListSecuritiesAccountsUseCase, RegisterSecuritiesAccountInput,
        RegisterSecuritiesAccountOutput, RegisterSecuritiesAccountUseCase,
        TestSecuritiesAccountConnectionUseCase, UpdateSecuritiesAccountInput,
        UpdateSecuritiesAccountOutput, UpdateSecuritiesAccountUseCase,
    },
    error::ApiError,
    infrastructure::DependencyContainer,
};

pub async fn list_accounts(
    State(container): State<DependencyContainer>,
) -> Result<Json<ListSecuritiesAccountsOutput>, ApiError> {
    Ok(Json(
        ListSecuritiesAccountsUseCase::new(container.account_repository()).execute()?,
    ))
}

pub async fn register_account(
    State(container): State<DependencyContainer>,
    Json(input): Json<RegisterSecuritiesAccountInput>,
) -> Result<(StatusCode, Json<RegisterSecuritiesAccountOutput>), ApiError> {
    let output =
        RegisterSecuritiesAccountUseCase::new(container.account_repository()).execute(input)?;
    Ok((StatusCode::CREATED, Json(output)))
}

pub async fn update_account(
    State(container): State<DependencyContainer>,
    Path(account_id): Path<String>,
    Json(mut input): Json<UpdateSecuritiesAccountInput>,
) -> Result<Json<UpdateSecuritiesAccountOutput>, ApiError> {
    input.account_identifier = account_id;
    let output =
        UpdateSecuritiesAccountUseCase::new(container.account_repository()).execute(input)?;
    Ok(Json(output))
}

pub async fn delete_account(
    State(container): State<DependencyContainer>,
    Path(account_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    DeleteSecuritiesAccountUseCase::new(container.account_repository()).execute(&account_id)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn test_connection(
    State(container): State<DependencyContainer>,
    Path(account_id): Path<String>,
) -> Result<Json<crate::application::use_cases::ConnectionTestOutput>, ApiError> {
    let output = TestSecuritiesAccountConnectionUseCase::new(
        container.account_repository(),
        container.browser_port(),
    )
    .execute(&account_id)
    .await?;
    Ok(Json(output))
}
