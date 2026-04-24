use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

use crate::{
    application::use_cases::{
        ListOperationLogsInput, ListOperationLogsOutput, ListOperationLogsUseCase,
    },
    infrastructure::DependencyContainer,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOperationLogsQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub event_type: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}

pub async fn list_logs(
    State(container): State<DependencyContainer>,
    Query(query): Query<ListOperationLogsQuery>,
) -> Result<Json<ListOperationLogsOutput>, crate::error::ApiError> {
    Ok(Json(
        ListOperationLogsUseCase::new(container.operation_log_repository())
            .execute(ListOperationLogsInput {
                start_date: query.start_date,
                end_date: query.end_date,
                event_type: query.event_type,
                cursor: query.cursor,
                limit: query.limit,
            })
            .await?,
    ))
}
