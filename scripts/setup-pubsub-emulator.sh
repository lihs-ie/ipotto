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
  local push_endpoint="${3:-}"

  local body="{\"topic\": \"projects/${PROJECT_ID}/topics/${topic}\"}"
  if [ -n "$push_endpoint" ]; then
    body="{\"topic\": \"projects/${PROJECT_ID}/topics/${topic}\", \"pushConfig\": {\"pushEndpoint\": \"${push_endpoint}\"}}"
  fi

  curl -s -X PUT "${BASE_URL}/subscriptions/${subscription}" \
    -H "Content-Type: application/json" \
    -d "$body" > /dev/null
  echo "Created subscription: ${subscription} -> ${topic}"
}

echo "Setting up Pub/Sub emulator topics and subscriptions..."

create_topic "ipo-job-trigger"
create_topic "ipo-info-updated"
create_topic "ipo-result-updated"
create_topic "ipo-notification"

create_subscription "ipo-info-fetch-sub" "ipo-job-trigger" "http://ipo-info-fetcher:8082/"
create_subscription "ipo-apply-sub" "ipo-job-trigger" "http://ipo-browser:8081/"
create_subscription "ipo-result-check-sub" "ipo-job-trigger" "http://ipo-result-checker:8083/"
create_subscription "ipo-info-updated-notify-sub" "ipo-info-updated" "http://ipo-api:8080/"
create_subscription "ipo-result-updated-notify-sub" "ipo-result-updated" "http://ipo-api:8080/"
create_subscription "ipo-notification-sub" "ipo-notification" "http://ipo-api:8080/"

echo "Pub/Sub emulator setup complete."
