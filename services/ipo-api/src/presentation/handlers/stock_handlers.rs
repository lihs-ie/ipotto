use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::{
    application::use_cases::{
        GetDashboardSummaryUseCase, GetIpoStockInput, GetIpoStockOutput, GetIpoStockUseCase,
        ListIpoStocksInput, ListIpoStocksOutput, ListIpoStocksUseCase,
    },
    infrastructure::DependencyContainer,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListIpoStocksQuery {
    pub status: Option<String>,
}

pub async fn list_stocks(
    State(container): State<DependencyContainer>,
    Query(query): Query<ListIpoStocksQuery>,
) -> Result<Json<ListIpoStocksOutput>, crate::error::ApiError> {
    Ok(Json(
        ListIpoStocksUseCase::new(container.stock_repository()).execute(ListIpoStocksInput {
            status_filter: query.status,
        })?,
    ))
}

pub async fn get_stock(
    State(container): State<DependencyContainer>,
    Path(stock_id): Path<String>,
) -> Result<Json<GetIpoStockOutput>, crate::error::ApiError> {
    Ok(Json(
        GetIpoStockUseCase::new(
            container.stock_repository(),
            container.application_repository(),
            container.account_repository(),
        )
        .execute(GetIpoStockInput {
            stock_identifier: stock_id,
        })?,
    ))
}

pub async fn get_dashboard(
    State(container): State<DependencyContainer>,
) -> Result<Json<crate::application::use_cases::DashboardSummaryOutput>, crate::error::ApiError> {
    Ok(Json(
        GetDashboardSummaryUseCase::new(
            container.stock_repository(),
            container.application_repository(),
            container.account_repository(),
        )
        .execute()?,
    ))
}
