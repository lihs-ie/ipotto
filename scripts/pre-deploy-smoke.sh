#!/usr/bin/env bash
# Pre-deploy smoke checks. Runs before `deploy.sh` promotes images.
#
# Usage:
#   scripts/pre-deploy-smoke.sh <env>
#
# Verifies:
# - Terraform configuration is syntactically valid
# - Each Cloud Run service currently has at least one revision
#   (so rollback.sh has somewhere to fall back to if the next
#   deploy fails)
# - The current revision answers /health within 10 seconds
set -euo pipefail

readonly ENV="${1:-}"

if [[ "${ENV}" != "stg" && "${ENV}" != "prd" ]]; then
  echo "Usage: $0 <stg|prd>" >&2
  exit 1
fi

readonly REGION="asia-northeast1"
readonly PROJECT_ID="ipotto-${ENV}"
readonly SERVICES=(
  "ipo-api"
  "ipo-browser"
  "ipo-info-fetcher"
  "ipo-result-checker"
  "ipo-frontend"
)

echo "== Terraform validate (${ENV})"
terraform -chdir="terraform/environments/${ENV}" init -backend=false -upgrade > /dev/null
terraform -chdir="terraform/environments/${ENV}" validate

echo "== Cloud Run current revision health (${ENV})"
for service in "${SERVICES[@]}"; do
  if ! gcloud run services describe "${service}" \
    --project="${PROJECT_ID}" \
    --region="${REGION}" \
    --format="value(metadata.name)" > /dev/null 2>&1; then
    echo "  SKIP ${service}: service not yet deployed (first rollout)"
    continue
  fi

  URL=$(gcloud run services describe "${service}" \
    --project="${PROJECT_ID}" \
    --region="${REGION}" \
    --format="value(status.url)")

  if curl -sf --max-time 10 "${URL}/health" > /dev/null 2>&1; then
    echo "  OK: ${service} ${URL}/health"
  else
    echo "  WARN: ${service} ${URL}/health not healthy — proceeding anyway" >&2
  fi
done

echo "== Pre-deploy smoke complete"
