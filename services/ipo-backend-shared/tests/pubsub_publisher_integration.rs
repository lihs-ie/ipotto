//! Integration tests for the production gRPC-backed Pub/Sub publisher.
//!
//! These tests require a Pub/Sub emulator reachable on `PUBSUB_EMULATOR_HOST`
//! with topics provisioned by `scripts/setup-pubsub-emulator.sh` (the CI
//! `rust-lint-test` job wires one up automatically). When the env var is
//! missing the tests become no-ops so `cargo test` on a developer laptop
//! without docker-compose still runs.

use std::{collections::HashMap, time::Duration};

use gcloud_pubsub::{
    client::{Client, ClientConfig},
    subscription::SubscriptionConfig,
};
use ipo_backend_shared::{
    acl::messaging::EventPublisherPort, infrastructure::messaging::PubSubEventPublisher,
};
use serde_json::json;
use uuid::Uuid;

fn emulator_available() -> bool {
    std::env::var("PUBSUB_EMULATOR_HOST").is_ok()
}

fn project_id() -> String {
    std::env::var("GCP_PROJECT").unwrap_or_else(|_| "ipotto-local".to_string())
}

async fn admin_client() -> Client {
    let config = ClientConfig {
        project_id: Some(project_id()),
        ..ClientConfig::default()
    };
    Client::new(config)
        .await
        .expect("Pub/Sub admin client for integration tests")
}

#[tokio::test]
async fn pubsub_publisher_emits_envelope_consumable_by_subscription() {
    if !emulator_available() {
        eprintln!("skipping: PUBSUB_EMULATOR_HOST not set");
        return;
    }

    // Each run uses a unique topic + subscription so parallel test runs and
    // previously-buffered messages cannot cross-talk.
    let suffix = Uuid::new_v4().simple().to_string();
    let topic_name = format!("ipo-publisher-integration-{suffix}");
    let subscription_name = format!("ipo-publisher-integration-sub-{suffix}");

    let admin = admin_client().await;
    let topic = admin.topic(&topic_name);
    topic
        .create(None, None)
        .await
        .expect("create integration topic");
    let subscription = admin.subscription(&subscription_name);
    subscription
        .create(
            topic.fully_qualified_name(),
            SubscriptionConfig::default(),
            None,
        )
        .await
        .expect("create integration subscription");

    let mut mapping = HashMap::new();
    mapping.insert("integration-event".to_string(), topic_name.clone());
    let publisher = PubSubEventPublisher::with_topic_mapping(
        "ipo-backend-shared-integration",
        project_id(),
        mapping,
    )
    .await
    .expect("build production publisher against emulator");

    let correlation_id = Uuid::new_v4();
    publisher
        .publish(
            "integration-event",
            "stock-integration",
            "IpoStock",
            json!({ "companyName": "統合テスト株式会社" }),
            Some(correlation_id),
        )
        .await
        .expect("publish against emulator");

    let received = tokio::time::timeout(Duration::from_secs(5), subscription.pull(1, None))
        .await
        .expect("pull did not time out")
        .expect("pull succeeds");
    assert!(
        !received.is_empty(),
        "emulator received the published message"
    );
    let body = String::from_utf8(received[0].message.data.clone()).expect("utf-8 body");
    assert!(
        body.contains("\"event_type\":\"integration-event\""),
        "envelope carries event_type: {body}"
    );
    assert!(
        body.contains("統合テスト株式会社"),
        "envelope carries payload: {body}"
    );
    for message in received {
        message.ack().await.ok();
    }

    subscription
        .delete(None)
        .await
        .expect("cleanup subscription");
    topic.delete(None).await.expect("cleanup topic");
}

#[tokio::test]
async fn pubsub_publisher_surfaces_missing_topic_mapping_as_domain_error() {
    if !emulator_available() {
        return;
    }
    let mut mapping = HashMap::new();
    mapping.insert("known-event".to_string(), "whatever".to_string());
    let publisher = PubSubEventPublisher::with_topic_mapping(
        "ipo-backend-shared-integration",
        project_id(),
        mapping,
    )
    .await
    .expect("build publisher");

    let result = publisher
        .publish(
            "missing-event",
            "stock-1",
            "IpoStock",
            json!({}),
            Some(Uuid::new_v4()),
        )
        .await;
    assert!(matches!(
        result,
        Err(ipo_backend_shared::errors::DomainError::PubSubPublishError { .. })
    ));
}
