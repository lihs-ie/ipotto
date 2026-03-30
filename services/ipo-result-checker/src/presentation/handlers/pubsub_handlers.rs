use axum::{extract::State, Json};

use crate::{
    application::use_cases::{CheckLotteryResultOutput, CheckLotteryResultUseCase},
    infrastructure::DependencyContainer,
};

pub async fn handle_check_message(
    State(container): State<DependencyContainer>,
) -> Result<Json<CheckLotteryResultOutput>, crate::error::ApiError> {
    Ok(Json(
        CheckLotteryResultUseCase::new(
            container.application_repository(),
            container.stock_repository(),
            container.account_repository(),
            container.browser_port(),
            container.event_publisher(),
            container.operation_log_repository(),
        )
        .execute()
        .await?,
    ))
}
