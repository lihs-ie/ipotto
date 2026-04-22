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
  local dead_letter_topic="${3-}"
  local max_delivery_attempts="${4:-5}"
  local ack_deadline_seconds="${5:-60}"

  local body
  if [[ -n "${dead_letter_topic}" ]]; then
    body=$(cat <<JSON
{
  "topic": "projects/${PROJECT_ID}/topics/${topic}",
  "ackDeadlineSeconds": ${ack_deadline_seconds},
  "deadLetterPolicy": {
    "deadLetterTopic": "projects/${PROJECT_ID}/topics/${dead_letter_topic}",
    "maxDeliveryAttempts": ${max_delivery_attempts}
  }
}
JSON
    )
  else
    body=$(cat <<JSON
{
  "topic": "projects/${PROJECT_ID}/topics/${topic}",
  "ackDeadlineSeconds": ${ack_deadline_seconds}
}
JSON
    )
  fi

  curl -s -X PUT "${BASE_URL}/subscriptions/${subscription}" \
    -H "Content-Type: application/json" \
    -d "${body}" > /dev/null
  echo "Created subscription: ${subscription} -> ${topic}${dead_letter_topic:+ (DLQ: ${dead_letter_topic}, max_attempts: ${max_delivery_attempts})}"
}

echo "Setting up Pub/Sub emulator topics and subscriptions..."

# Production topics
create_topic "ipo-job-trigger"
create_topic "ipo-info-updated"
create_topic "ipo-result-updated"
create_topic "ipo-notification"

# Shared dead-letter topic. Subscriptions whose max_delivery_attempts
# are exhausted ship their payloads here so operators can inspect them
# without losing the original message.
create_topic "ipo-dead-letter"

# Primary subscriptions, each wired to the shared DLQ.
create_subscription "ipo-info-fetch-sub"            "ipo-job-trigger"     "ipo-dead-letter" 5 60
create_subscription "ipo-apply-sub"                 "ipo-job-trigger"     "ipo-dead-letter" 5 60
create_subscription "ipo-result-check-sub"          "ipo-job-trigger"     "ipo-dead-letter" 5 60
create_subscription "ipo-info-updated-notify-sub"   "ipo-info-updated"    "ipo-dead-letter" 5 60
create_subscription "ipo-result-updated-notify-sub" "ipo-result-updated"  "ipo-dead-letter" 5 60
create_subscription "ipo-notification-sub"          "ipo-notification"    "ipo-dead-letter" 5 60

# DLQ monitor subscription so local pull-based smoke checks can surface
# dead-lettered messages without creating one ad-hoc each run.
create_subscription "ipo-dead-letter-sub" "ipo-dead-letter"

echo "Pub/Sub emulator setup complete."
