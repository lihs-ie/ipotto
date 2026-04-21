// Phase 3 Sprint 6.5 — SEV1 notification publisher. Forwards broker
// automation failures (today: 2FA image-auth failures; PR #d+ may add
// apply / result-check failures) to ipo-api's
// /internal/pubsub/ipo-notification endpoint so the domain dispatcher
// fans them out to LINE / Email / Slack per the operator's settings.
//
// The publisher is intentionally best-effort: we log any publish
// failure but never throw out of the caller so that the browser
// automation path can still return a structured failure response to
// ipo-api. Re-delivery semantics are the responsibility of the
// dispatcher on the ipo-api side.

const DEFAULT_IPO_API_BASE_URL = "http://ipo-api:8080";
const DEFAULT_PUBLISH_PATH = "/internal/pubsub/ipo-notification";

export type OperationErrorPayload = {
  readonly serviceName: string;
  readonly operationType: string;
  readonly errorMessage: string;
  readonly occurredAt?: Date;
};

export type NotificationPublisher = {
  readonly publishOperationError: (
    payload: OperationErrorPayload,
  ) => Promise<void>;
};

export type NotificationPublisherOptions = {
  readonly baseUrl?: string;
  readonly fetcher?: typeof fetch;
};

export function createNotificationPublisher(
  options: NotificationPublisherOptions = {},
): NotificationPublisher {
  const baseUrl =
    options.baseUrl ??
    process.env["IPO_API_BASE_URL"] ??
    DEFAULT_IPO_API_BASE_URL;
  const fetcher = options.fetcher ?? fetch;
  const url = `${baseUrl}${DEFAULT_PUBLISH_PATH}`;

  return {
    async publishOperationError(payload) {
      const occurredAt = payload.occurredAt ?? new Date();
      const body = {
        eventType: "OperationErrorOccurred",
        payload: {
          service_name: payload.serviceName,
          operation_type: payload.operationType,
          error_message: payload.errorMessage,
          occurred_at: occurredAt.toISOString(),
        },
      };
      try {
        const response = await fetcher(url, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(body),
        });
        if (!response.ok) {
          console.error(
            `OperationErrorOccurred publish returned non-2xx status=${response.status}`,
          );
        }
      } catch (error) {
        console.error(
          "OperationErrorOccurred publish failed",
          error,
        );
      }
    },
  };
}
