use axum::{extract::State, http::StatusCode, Json};
use ipo_backend_shared::{
    acl::notification::NotificationEvent,
    domain::notification::NotificationEventType,
    events::{
        ApplicationCompleted, ApplicationFailed, ImageAuthenticationFailed, IpoInfoUpdated,
        LotteryResultConfirmed, OperationErrorOccurred,
    },
};
use serde::Deserialize;

use crate::{
    application::use_cases::{
        DispatchNotificationUseCase, GetNotificationSettingOutput, GetNotificationSettingUseCase,
        UpdateNotificationSettingInput, UpdateNotificationSettingOutput,
        UpdateNotificationSettingUseCase,
    },
    error::ApiError,
    infrastructure::DependencyContainer,
};

pub async fn get_setting(
    State(container): State<DependencyContainer>,
) -> Result<Json<GetNotificationSettingOutput>, ApiError> {
    Ok(Json(
        GetNotificationSettingUseCase::new(container.notification_setting_repository())
            .execute()?,
    ))
}

pub async fn update_setting(
    State(container): State<DependencyContainer>,
    Json(input): Json<UpdateNotificationSettingInput>,
) -> Result<Json<UpdateNotificationSettingOutput>, ApiError> {
    Ok(Json(
        UpdateNotificationSettingUseCase::new(container.notification_setting_repository())
            .execute(input)?,
    ))
}

pub async fn handle_ipo_info_updated(
    State(container): State<DependencyContainer>,
    Json(event): Json<IpoInfoUpdated>,
) -> Result<StatusCode, ApiError> {
    DispatchNotificationUseCase::new(
        container.notification_setting_repository(),
        container.notification_ports(),
    )
    .execute(
        NotificationEvent::IpoInfoUpdated(event),
        NotificationEventType::StockUpdated,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn handle_lottery_result_updated(
    State(container): State<DependencyContainer>,
    Json(event): Json<LotteryResultConfirmed>,
) -> Result<StatusCode, ApiError> {
    let event_type = match event.lottery_result {
        ipo_backend_shared::domain::application::LotteryResult::Won => {
            NotificationEventType::LotteryResultWon
        }
        ipo_backend_shared::domain::application::LotteryResult::Lost
        | ipo_backend_shared::domain::application::LotteryResult::Alternate => {
            NotificationEventType::LotteryResultLost
        }
    };
    DispatchNotificationUseCase::new(
        container.notification_setting_repository(),
        container.notification_ports(),
    )
    .execute(NotificationEvent::LotteryResultConfirmed(event), event_type)
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericNotificationPushRequest {
    pub event_type: String,
    pub payload: serde_json::Value,
}

pub async fn handle_generic_notification(
    State(container): State<DependencyContainer>,
    Json(request): Json<GenericNotificationPushRequest>,
) -> Result<StatusCode, ApiError> {
    let (event, event_type) = parse_notification_request(request)?;
    DispatchNotificationUseCase::new(
        container.notification_setting_repository(),
        container.notification_ports(),
    )
    .execute(event, event_type)
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

fn parse_notification_request(
    request: GenericNotificationPushRequest,
) -> Result<(NotificationEvent, NotificationEventType), ApiError> {
    match request.event_type.as_str() {
        "ApplicationCompleted" => Ok((
            NotificationEvent::ApplicationCompleted(
                serde_json::from_value::<ApplicationCompleted>(request.payload)
                    .map_err(|error| ApiError::validation(error.to_string(), "payload"))?,
            ),
            NotificationEventType::ApplicationCompleted,
        )),
        "ApplicationFailed" => Ok((
            NotificationEvent::ApplicationFailed(
                serde_json::from_value::<ApplicationFailed>(request.payload)
                    .map_err(|error| ApiError::validation(error.to_string(), "payload"))?,
            ),
            NotificationEventType::OperationError,
        )),
        "LotteryResultConfirmed" => {
            let event = serde_json::from_value::<LotteryResultConfirmed>(request.payload)
                .map_err(|error| ApiError::validation(error.to_string(), "payload"))?;
            let event_type = match event.lottery_result {
                ipo_backend_shared::domain::application::LotteryResult::Won => {
                    NotificationEventType::LotteryResultWon
                }
                ipo_backend_shared::domain::application::LotteryResult::Lost
                | ipo_backend_shared::domain::application::LotteryResult::Alternate => {
                    NotificationEventType::LotteryResultLost
                }
            };
            Ok((NotificationEvent::LotteryResultConfirmed(event), event_type))
        }
        "IpoInfoUpdated" => Ok((
            NotificationEvent::IpoInfoUpdated(
                serde_json::from_value::<IpoInfoUpdated>(request.payload)
                    .map_err(|error| ApiError::validation(error.to_string(), "payload"))?,
            ),
            NotificationEventType::StockUpdated,
        )),
        "OperationErrorOccurred" => Ok((
            NotificationEvent::OperationErrorOccurred(
                serde_json::from_value::<OperationErrorOccurred>(request.payload)
                    .map_err(|error| ApiError::validation(error.to_string(), "payload"))?,
            ),
            NotificationEventType::OperationError,
        )),
        "ImageAuthenticationFailed" => Ok((
            NotificationEvent::ImageAuthenticationFailed(
                serde_json::from_value::<ImageAuthenticationFailed>(request.payload)
                    .map_err(|error| ApiError::validation(error.to_string(), "payload"))?,
            ),
            NotificationEventType::OperationError,
        )),
        other => Err(ApiError::validation(
            format!("unsupported event type: {other}"),
            "eventType",
        )),
    }
}
