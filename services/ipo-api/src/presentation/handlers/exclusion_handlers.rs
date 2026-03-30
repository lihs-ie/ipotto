use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    application::use_cases::{
        ListExclusionsUseCase, RegisterExclusionInput, RegisterExclusionUseCase,
        RemoveExclusionUseCase,
    },
    error::ApiError,
    infrastructure::DependencyContainer,
};

pub async fn list_exclusions(
    State(container): State<DependencyContainer>,
) -> Result<Json<crate::application::use_cases::ListExclusionsOutput>, ApiError> {
    Ok(Json(
        ListExclusionsUseCase::new(container.exclusion_repository()).execute()?,
    ))
}

pub async fn register_exclusion(
    State(container): State<DependencyContainer>,
    Json(input): Json<RegisterExclusionInput>,
) -> Result<
    (
        StatusCode,
        Json<crate::application::use_cases::RegisterExclusionOutput>,
    ),
    ApiError,
> {
    let output = RegisterExclusionUseCase::new(container.exclusion_repository()).execute(input)?;
    Ok((StatusCode::CREATED, Json(output)))
}

pub async fn remove_exclusion(
    State(container): State<DependencyContainer>,
    Path(exclusion_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    RemoveExclusionUseCase::new(container.exclusion_repository()).execute(&exclusion_id)?;
    Ok(StatusCode::NO_CONTENT)
}
