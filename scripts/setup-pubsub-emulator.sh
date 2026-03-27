#!/usr/bin/env bash
set -euo pipefail

PUBSUB_HOST="${PUBSUB_EMULATOR_HOST:-localhost:8085}"
PROJECT_ID="${GCP_PROJECT:-ipotto-local}"
BASE_URL="http://${PUBSUB_HOST}/v1/projects/${PROJECT_ID}"

create_topic() {
  local topic="$1"
  curl -s -X PUT "${BASE_URL}/topics/${topic}" > /dev/null
  echo "Created topic: ${topic}"
}

create_subscription() {
  local subscription="$1"
  local topic="$2"

  curl -s -X PUT "${BASE_URL}/subscriptions/${subscription}" \
    -H "Content-Type: application/json" \
    -d "{\"topic\": \"projects/${PROJECT_ID}/topics/${topic}\"}" > /dev/null
  echo "Created subscription: ${subscription} -> ${topic}"
}

echo "Setting up Pub/Sub emulator topics and subscriptions..."

create_topic "ipo-job-trigger"
create_topic "ipo-info-updated"
create_topic "ipo-result-updated"
create_topic "ipo-notification"

create_subscription "ipo-info-fetch-sub" "ipo-job-trigger"
create_subscription "ipo-apply-sub" "ipo-job-trigger"
create_subscription "ipo-result-check-sub" "ipo-job-trigger"
create_subscription "ipo-info-updated-notify-sub" "ipo-info-updated"
create_subscription "ipo-result-updated-notify-sub" "ipo-result-updated"
create_subscription "ipo-notification-sub" "ipo-notification"

echo "Pub/Sub emulator setup complete."
