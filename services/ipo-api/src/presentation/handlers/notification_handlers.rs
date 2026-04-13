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

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, sync::Arc};

    use async_trait::async_trait;
    use axum::{extract::State, Json};
    use chrono::Utc;
    use ipo_backend_shared::{
        acl::{browser::BrokerBrowserPort, notification::NotificationPort},
        domain::{
            account::{AccountCredential, ConnectionTestResult},
            notification::NotificationSettingRepository,
            stock::IpoStock,
        },
        errors::DomainError,
        infrastructure::firestore::repositories::FirestoreNotificationSettingRepository,
    };
    use tokio::sync::Mutex;

    use crate::application::use_cases::ChannelInput;
    use crate::infrastructure::NotificationPortRegistry;

    use super::{
        get_setting, handle_generic_notification, parse_notification_request, update_setting,
        GenericNotificationPushRequest, UpdateNotificationSettingInput,
    };

    #[derive(Debug, Default)]
    struct RecordingPort {
        calls: Mutex<usize>,
    }

    #[derive(Debug)]
    struct DummyBrowserPort;

    #[async_trait]
    impl BrokerBrowserPort for DummyBrowserPort {
        async fn fetch_ipo_stocks(
            &self,
        ) -> Result<Vec<ipo_backend_shared::acl::scraping::ScrapedStock>, DomainError> {
            Ok(Vec::new())
        }

        async fn test_connection(
            &self,
            _credential: &AccountCredential,
        ) -> Result<ConnectionTestResult, DomainError> {
            Ok(ConnectionTestResult::new(true, "ok", Utc::now()))
        }

        async fn check_lottery_result(
            &self,
            _credential: &AccountCredential,
            _stock: &IpoStock,
        ) -> Result<Option<ipo_backend_shared::domain::application::LotteryResult>, DomainError>
        {
            Ok(None)
        }
    }

    #[async_trait]
    impl NotificationPort for RecordingPort {
        async fn send(
            &self,
            _event: &ipo_backend_shared::acl::notification::NotificationEvent,
            _destination: &ipo_backend_shared::domain::notification::ChannelDestination,
        ) -> Result<(), DomainError> {
            *self.calls.lock().await += 1;
            Ok(())
        }

        fn validate_destination(
            &self,
            _destination: &ipo_backend_shared::domain::notification::ChannelDestination,
        ) -> Result<(), DomainError> {
            Ok(())
        }
    }

    fn build_container() -> crate::infrastructure::DependencyContainer {
        crate::infrastructure::DependencyContainer::from_components(
            Arc::new(
                ipo_backend_shared::infrastructure::firestore::repositories::FirestoreIpoStockRepository::new(),
            ),
            Arc::new(
                ipo_backend_shared::infrastructure::firestore::repositories::FirestoreExclusionRepository::new(),
            ),
            Arc::new(
                ipo_backend_shared::infrastructure::firestore::repositories::FirestoreLotteryApplicationRepository::new(),
            ),
            Arc::new(
                ipo_backend_shared::infrastructure::firestore::repositories::FirestoreSecuritiesAccountRepository::new(
                    ipo_backend_shared::infrastructure::secrets::InMemoryCredentialStore::new(),
                ),
            ),
            Arc::new(FirestoreNotificationSettingRepository::new())
                as Arc<dyn NotificationSettingRepository + Send + Sync>,
            Arc::new(
                ipo_backend_shared::infrastructure::firestore::repositories::FirestoreOperationLogRepository::new(),
            ),
            Arc::new(DummyBrowserPort),
            Arc::new(NotificationPortRegistry::new(
                Arc::new(RecordingPort::default()),
                Arc::new(RecordingPort::default()),
                Arc::new(RecordingPort::default()),
            )),
        )
        .expect("container")
    }

    #[tokio::test]
    async fn updates_and_gets_notification_settings() {
        let container = build_container();
        let mut destination = BTreeMap::new();
        destination.insert("address".to_string(), "notify@example.com".to_string());
        let mut subscriptions = BTreeMap::new();
        subscriptions.insert("ApplicationCompleted".to_string(), true);

        let Json(updated) = update_setting(
            State(container.clone()),
            Json(UpdateNotificationSettingInput {
                enabled: true,
                channels: vec![ChannelInput {
                    channel_type: "Email".to_string(),
                    destination,
                    enabled: true,
                    subscriptions,
                }],
            }),
        )
        .await
        .expect("update");
        assert_eq!(updated.channel_count, 1);

        let Json(setting) = get_setting(State(container)).await.expect("get");
        assert_eq!(setting.channels.len(), 1);
    }

    #[tokio::test]
    async fn dispatches_a_generic_notification_event() {
        let container = build_container();
        let _ = update_setting(
            State(container.clone()),
            Json(UpdateNotificationSettingInput {
                enabled: true,
                channels: vec![ChannelInput {
                    channel_type: "Email".to_string(),
                    destination: BTreeMap::from([(
                        "address".to_string(),
                        "notify@example.com".to_string(),
                    )]),
                    enabled: true,
                    subscriptions: BTreeMap::from([(
                        "ApplicationCompleted".to_string(),
                        true,
                    )]),
                }],
            }),
        )
        .await
        .expect("update");

        let status = handle_generic_notification(
            State(container),
            Json(GenericNotificationPushRequest {
                event_type: "ApplicationCompleted".to_string(),
                payload: serde_json::json!({
                    "identifier": ipo_backend_shared::domain::application::ApplicationIdentifier::generate(),
                    "stock": ipo_backend_shared::domain::stock::StockIdentifier::generate(),
                    "securities_account": ipo_backend_shared::domain::account::SecuritiesAccountIdentifier::generate(),
                    "applied_shares": 100,
                    "applied_price": 1200,
                    "applied_at": Utc::now(),
                }),
            }),
        )
        .await
        .expect("dispatch");

        assert_eq!(status, axum::http::StatusCode::NO_CONTENT);
    }

    #[test]
    fn validates_unknown_generic_notification_types() {
        let error = parse_notification_request(GenericNotificationPushRequest {
            event_type: "UnknownEvent".to_string(),
            payload: serde_json::json!({}),
        })
        .expect_err("unsupported type");

        let response = axum::response::IntoResponse::into_response(error);
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }
}
