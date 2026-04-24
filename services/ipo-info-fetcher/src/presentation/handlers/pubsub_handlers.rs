use axum::{extract::State, Json};

use crate::{
    application::use_cases::{FetchIpoStocksOutput, FetchIpoStocksUseCase},
    infrastructure::DependencyContainer,
};

pub async fn handle_fetch_message(
    State(container): State<DependencyContainer>,
) -> Result<Json<FetchIpoStocksOutput>, crate::error::ApiError> {
    Ok(Json(
        FetchIpoStocksUseCase::new(
            container.stock_repository(),
            container.scraper(),
            container.event_publisher(),
            container.operation_log_repository(),
        )
        .execute()
        .await?,
    ))
}
