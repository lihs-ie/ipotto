use axum::{extract::State, Json};
use chrono::{NaiveDate, Utc};
use serde::Deserialize;

use crate::{
    application::use_cases::{ApplyForLotteryInput, ApplyForLotteryOutput, ApplyForLotteryUseCase},
    error::ApiError,
    infrastructure::DependencyContainer,
};

/// Pub/Sub trigger payload. Cloud Scheduler posts a JSON body of the
/// form `{"targetDate": "YYYY-MM-DD"}`; when the body is empty or the
/// field is absent the handler defaults to the current UTC date so
/// operators can dispatch ad-hoc runs with an empty body.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ApplyForLotteryTriggerPayload {
    pub target_date: Option<NaiveDate>,
}

pub async fn handle_apply_message(
    State(container): State<DependencyContainer>,
    payload: Option<Json<ApplyForLotteryTriggerPayload>>,
) -> Result<Json<ApplyForLotteryOutput>, ApiError> {
    let target_date = payload
        .and_then(|Json(body)| body.target_date)
        .unwrap_or_else(|| Utc::now().date_naive());

    Ok(Json(
        ApplyForLotteryUseCase::new(
            container.application_repository(),
            container.stock_repository(),
            container.account_repository(),
            container.exclusion_repository(),
            container.browser_port(),
            container.event_publisher(),
            container.operation_log_repository(),
        )
        .execute(ApplyForLotteryInput { target_date })
        .await?,
    ))
}
