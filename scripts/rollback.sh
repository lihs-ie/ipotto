#!/usr/bin/env bash
# Roll back a Cloud Run service to its previous revision.
#
# Usage:
#   scripts/rollback.sh <env> <service>
#
# - Identifies the two most recent revisions by creation time
# - Shifts 100% of traffic back to the previous (second-most-recent)
#   revision
# - Fails fast if only one revision exists (nothing to roll back to)
set -euo pipefail

readonly ENV="${1:-}"
readonly SERVICE="${2:-}"

if [[ "${ENV}" != "stg" && "${ENV}" != "prd" ]]; then
  echo "Usage: $0 <stg|prd> <service>" >&2
  exit 1
fi

if [[ -z "${SERVICE}" ]]; then
  echo "Usage: $0 <stg|prd> <service>" >&2
  exit 1
fi

readonly REGION="asia-northeast1"
readonly PROJECT_ID="ipotto-${ENV}"

echo "Rolling back ${SERVICE} in ${ENV}"

REVISIONS=$(gcloud run revisions list \
  --project="${PROJECT_ID}" \
  --region="${REGION}" \
  --service="${SERVICE}" \
  --format="value(metadata.name)" \
  --sort-by="~metadata.creationTimestamp" \
  --limit=2)

REV_COUNT=$(echo "${REVISIONS}" | grep -c . || true)
if [[ "${REV_COUNT}" -lt 2 ]]; then
  echo "Only ${REV_COUNT} revision(s) exist for ${SERVICE} — cannot roll back" >&2
  exit 1
fi

PREVIOUS=$(echo "${REVISIONS}" | sed -n '2p')

echo "Shifting 100% traffic to ${PREVIOUS}"
gcloud run services update-traffic "${SERVICE}" \
  --project="${PROJECT_ID}" \
  --region="${REGION}" \
  --to-revisions="${PREVIOUS}=100" \
  --quiet

echo "Rollback complete. Verifying /health"
URL=$(gcloud run services describe "${SERVICE}" \
  --project="${PROJECT_ID}" \
  --region="${REGION}" \
  --format="value(status.url)")

for attempt in {1..6}; do
  if curl -sf "${URL}/health" > /dev/null 2>&1; then
    echo "OK: ${SERVICE} healthy at ${URL} (previous revision)"
    exit 0
  fi
  sleep 5
done

echo "WARNING: ${SERVICE} still unhealthy after rollback. Manual intervention required." >&2
exit 1
