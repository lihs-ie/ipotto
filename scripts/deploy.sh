#!/usr/bin/env bash
# IPOtto production deployment helper.
#
# Usage:
#   scripts/deploy.sh <env> [service1 service2 ...]
#
# - <env> is `stg` or `prd`
# - Services default to all 5 Cloud Run services
# - Health checks each service /health endpoint after deploy
# - On failure, calls rollback.sh to restore previous revision
set -euo pipefail

readonly ENV="${1:-}"
shift || true

if [[ "${ENV}" != "stg" && "${ENV}" != "prd" ]]; then
  echo "Usage: $0 <stg|prd> [service1 service2 ...]" >&2
  exit 1
fi

readonly DEFAULT_SERVICES=(
  "ipo-api"
  "ipo-browser"
  "ipo-info-fetcher"
  "ipo-result-checker"
  "ipo-frontend"
)
readonly SERVICES=("${@:-${DEFAULT_SERVICES[@]}}")
readonly REGION="asia-northeast1"
readonly PROJECT_ID="ipotto-${ENV}"

echo "Deploying to ${ENV} (project=${PROJECT_ID}, region=${REGION})"
echo "Services: ${SERVICES[*]}"

echo "== Pre-deploy auth check"
if ! gcloud auth list --filter="status:ACTIVE" --format="value(account)" | grep -q .; then
  echo "No active gcloud account. Run 'gcloud auth login' first." >&2
  exit 1
fi

echo "== Pre-deploy smoke"
bash "$(dirname "$0")/pre-deploy-smoke.sh" "${ENV}"

FAILED_SERVICES=()
for service in "${SERVICES[@]}"; do
  echo "== Deploying ${service}"
  if ! gcloud run deploy "${service}" \
    --project="${PROJECT_ID}" \
    --region="${REGION}" \
    --platform=managed \
    --quiet; then
    echo "Deploy failed for ${service}" >&2
    FAILED_SERVICES+=("${service}")
    continue
  fi

  echo "== Post-deploy health check for ${service}"
  URL=$(gcloud run services describe "${service}" \
    --project="${PROJECT_ID}" \
    --region="${REGION}" \
    --format="value(status.url)")

  OK=false
  for attempt in {1..12}; do
    if curl -sf "${URL}/health" > /dev/null 2>&1; then
      OK=true
      break
    fi
    echo "  Attempt ${attempt}/12 for ${service} — not yet healthy"
    sleep 5
  done

  if ! ${OK}; then
    echo "Health check failed for ${service} at ${URL}/health" >&2
    FAILED_SERVICES+=("${service}")
  else
    echo "  OK: ${service} healthy at ${URL}"
  fi
done

if [[ ${#FAILED_SERVICES[@]} -gt 0 ]]; then
  echo "== Rollback triggered for: ${FAILED_SERVICES[*]}"
  for service in "${FAILED_SERVICES[@]}"; do
    bash "$(dirname "$0")/rollback.sh" "${ENV}" "${service}" || true
  done
  exit 1
fi

echo "== All ${#SERVICES[@]} services deployed and healthy"
