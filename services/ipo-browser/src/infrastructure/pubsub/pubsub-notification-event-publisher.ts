import { PubSub } from "@google-cloud/pubsub";

import type { NotificationEventPublisher } from "../../application/use-cases/apply-for-lottery-use-case.js";
import type { NotificationEventEnvelope } from "../../domain/notification-event.js";

interface PubSubEnvelope {
  readonly messageId: string;
  readonly eventType: string;
  readonly aggregateId: string;
  readonly aggregateType: string;
  readonly payload: Record<string, unknown>;
  readonly metadata: {
    readonly timestamp: string;
    readonly version: number;
    readonly correlationId: string;
    readonly serviceName: string;
  };
}

/**
 * Pub/Sub backed event publisher for ipo-browser notifications.
 */
export class PubSubNotificationEventPublisher
  implements NotificationEventPublisher
{
  /**
   * Creates the publisher.
   */
  public constructor(
    private readonly pubsub: PubSub,
    private readonly topicName: string,
  ) {}

  /**
   * Publishes an event to the notification topic.
   */
  public async publish(event: NotificationEventEnvelope): Promise<void> {
    const body: PubSubEnvelope = {
      messageId: crypto.randomUUID(),
      eventType: event.eventType,
      aggregateId: event.aggregateId,
      aggregateType: event.aggregateType,
      payload: event.payload,
      metadata: {
        timestamp: new Date().toISOString(),
        version: 1,
        correlationId: crypto.randomUUID(),
        serviceName: "ipo-browser",
      },
    };

    await this.pubsub
      .topic(this.topicName)
      .publishMessage({ json: body });
  }
}
