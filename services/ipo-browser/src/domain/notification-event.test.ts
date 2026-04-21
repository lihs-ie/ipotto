import { describe, expect, it } from "vitest";

import {
  OPERATION_LOG_EVENT_TYPES,
  type ApplicationCompletedPayload,
  type ApplicationFailedPayload,
  type ImageAuthenticationFailedPayload,
  type NotificationEventEnvelope,
  type OperationErrorOccurredPayload,
  type OperationLogEntry,
} from "./notification-event.js";

describe("notification-event contracts", () => {
  it("exposes the supported operation log event type constants", () => {
    expect(OPERATION_LOG_EVENT_TYPES).toEqual({
      APPLY_LOTTERY: "ApplyLottery",
    });
  });

  it("keeps the notification payload shapes used across services", () => {
    const completed: ApplicationCompletedPayload = {
      identifier: "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      stock: "stock-1",
      securities_account: "account-1",
      applied_shares: 100,
      applied_price: 600,
      applied_at: "2026-04-13T00:00:00.000Z",
    };
    const failed: ApplicationFailedPayload = {
      identifier: "stock-1",
      stock: "stock-1",
      securities_account: "account-1",
      error_message: "申し込みに失敗しました",
      failed_at: "2026-04-13T00:00:00.000Z",
    };
    const imageFailure: ImageAuthenticationFailedPayload = {
      securities_account: "account-1",
      failure_reason: "認証メールの取得に失敗しました",
      attempt_count: 1,
      occurred_at: "2026-04-13T00:00:00.000Z",
    };
    const operationError: OperationErrorOccurredPayload = {
      service_name: "ipo-browser",
      operation_type: "apply_lottery",
      error_message: "workflow failed",
      occurred_at: "2026-04-13T00:00:00.000Z",
    };

    const envelope: NotificationEventEnvelope = {
      eventType: "ApplicationCompleted",
      aggregateId: completed.identifier,
      aggregateType: "LotteryApplication",
      payload: completed,
    };
    const log: OperationLogEntry = {
      identifier: "log-1",
      application: completed.identifier,
      eventType: OPERATION_LOG_EVENT_TYPES.APPLY_LOTTERY,
      serviceName: "ipo-browser",
      status: "Success",
      message: "IPO 抽選申込が完了しました",
      errorMessage: null,
      executedAt: completed.applied_at,
    };

    expect(envelope.aggregateType).toBe("LotteryApplication");
    expect(failed.error_message).toContain("失敗");
    expect(imageFailure.attempt_count).toBe(1);
    expect(operationError.service_name).toBe("ipo-browser");
    expect(log.eventType).toBe(OPERATION_LOG_EVENT_TYPES.APPLY_LOTTERY);
  });
});
