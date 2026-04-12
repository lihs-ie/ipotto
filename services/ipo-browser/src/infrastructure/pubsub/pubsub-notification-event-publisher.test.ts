import { describe, expect, it, vi } from "vitest";
import type { PubSub } from "@google-cloud/pubsub";

import { generateUlid } from "../../domain/generate-ulid.js";
import type { NotificationEventEnvelope } from "../../domain/notification-event.js";
import { PubSubNotificationEventPublisher } from "./pubsub-notification-event-publisher.js";

describe("PubSubNotificationEventPublisher", () => {
  it("publishes an envelope with UUID metadata while preserving ULID aggregate identifiers", async () => {
    const publishMessage = vi.fn(
      async (message: { readonly json: unknown }) => {
        void message;
        return "message-id";
      },
    );
    const topic = vi.fn(() => ({ publishMessage }));
    const publisher = new PubSubNotificationEventPublisher(
      { topic } as unknown as PubSub,
      "ipo-notification",
    );
    const event: NotificationEventEnvelope = {
      eventType: "ApplicationCompleted",
      aggregateId: generateUlid(),
      aggregateType: "LotteryApplication",
      payload: {
        identifier: generateUlid(),
        stock: generateUlid(),
        securities_account: generateUlid(),
        applied_shares: 100,
        applied_price: 1400,
        applied_at: "2026-04-10T00:00:00.000Z",
      },
    };

    await publisher.publish(event);

    expect(topic).toHaveBeenCalledWith("ipo-notification");
    const firstCall = publishMessage.mock.calls[0];
    expect(firstCall).toBeDefined();
    const body = firstCall?.[0].json as {
      readonly messageId: string;
      readonly aggregateId: string;
      readonly payload: {
        readonly identifier: string;
        readonly stock: string;
        readonly securities_account: string;
      };
      readonly metadata: {
        readonly correlationId: string;
        readonly serviceName: string;
      };
    };
    expect(body.aggregateId).toBe(event.aggregateId);
    expect(body.payload.identifier).toBe(event.payload.identifier);
    expect(body.payload.stock).toBe(event.payload.stock);
    expect(body.payload.securities_account).toBe(event.payload.securities_account);
    expect(body.metadata.serviceName).toBe("ipo-browser");
    expect(body.messageId).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/iu,
    );
    expect(body.metadata.correlationId).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/iu,
    );
    expect(body.aggregateId).toMatch(/^[0-9A-HJKMNP-TV-Z]{26}$/u);
  });

  it("preserves shared aggregate-type policy across browser notification events", async () => {
    const publishMessage = vi.fn(
      async (message: { readonly json: unknown }) => {
        void message;
        return "message-id";
      },
    );
    const topic = vi.fn(() => ({ publishMessage }));
    const publisher = new PubSubNotificationEventPublisher(
      { topic } as unknown as PubSub,
      "ipo-notification",
    );
    const applicationFailedEvent: NotificationEventEnvelope = {
      eventType: "ApplicationFailed",
      aggregateId: generateUlid(),
      aggregateType: "IpoStock",
      payload: {
        identifier: generateUlid(),
        stock: generateUlid(),
        securities_account: generateUlid(),
        error_message: "IPO抽選の申し込みに失敗しました",
        failed_at: "2026-04-10T00:00:00.000Z",
      },
    };
    const imageAuthenticationFailedEvent: NotificationEventEnvelope = {
      eventType: "ImageAuthenticationFailed",
      aggregateId: generateUlid(),
      aggregateType: "SecuritiesAccount",
      payload: {
        securities_account: generateUlid(),
        failure_reason: "画像認証に失敗しました",
        attempt_count: 1,
        occurred_at: "2026-04-10T00:00:00.000Z",
      },
    };

    await publisher.publish(applicationFailedEvent);
    await publisher.publish(imageAuthenticationFailedEvent);

    const applicationCall = publishMessage.mock.calls[0];
    expect(applicationCall).toBeDefined();
    const applicationBody = applicationCall?.[0].json as {
      readonly aggregateId: string;
      readonly aggregateType: string;
      readonly metadata: { readonly correlationId: string };
      readonly payload: {
        readonly identifier: string;
        readonly stock: string;
        readonly securities_account: string;
      };
    };
    const imageAuthenticationCall = publishMessage.mock.calls[1];
    expect(imageAuthenticationCall).toBeDefined();
    const imageAuthenticationBody = imageAuthenticationCall?.[0].json as {
      readonly aggregateId: string;
      readonly aggregateType: string;
      readonly metadata: { readonly correlationId: string };
      readonly payload: { readonly securities_account: string };
    };

    expect(applicationBody.aggregateType).toBe("IpoStock");
    expect(applicationBody.aggregateId).toMatch(/^[0-9A-HJKMNP-TV-Z]{26}$/u);
    expect(applicationBody.payload.identifier).toMatch(/^[0-9A-HJKMNP-TV-Z]{26}$/u);
    expect(applicationBody.payload.stock).toMatch(/^[0-9A-HJKMNP-TV-Z]{26}$/u);
    expect(applicationBody.payload.securities_account).toMatch(
      /^[0-9A-HJKMNP-TV-Z]{26}$/u,
    );
    expect(applicationBody.metadata.correlationId).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/iu,
    );

    expect(imageAuthenticationBody.aggregateType).toBe("SecuritiesAccount");
    expect(imageAuthenticationBody.aggregateId).toMatch(/^[0-9A-HJKMNP-TV-Z]{26}$/u);
    expect(imageAuthenticationBody.payload.securities_account).toMatch(
      /^[0-9A-HJKMNP-TV-Z]{26}$/u,
    );
    expect(imageAuthenticationBody.metadata.correlationId).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/iu,
    );
  });
});
