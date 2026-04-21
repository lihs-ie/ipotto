/**
 * Payload for successful application events.
 */
export interface ApplicationCompletedPayload {
  readonly identifier: string;
  readonly stock: string;
  readonly securities_account: string;
  readonly applied_shares: number;
  readonly applied_price: number;
  readonly applied_at: string;
}

/**
 * Payload for application failures after broker submission started.
 */
export interface ApplicationFailedPayload {
  readonly identifier: string;
  readonly stock: string;
  readonly securities_account: string;
  readonly error_message: string;
  readonly failed_at: string;
}

/**
 * Payload for image authentication related failures.
 */
export interface ImageAuthenticationFailedPayload {
  readonly securities_account: string;
  readonly failure_reason: string;
  readonly attempt_count: number;
  readonly occurred_at: string;
}

/**
 * Payload for operational workflow failures.
 */
export interface OperationErrorOccurredPayload {
  readonly service_name: string;
  readonly operation_type: string;
  readonly error_message: string;
  readonly occurred_at: string;
}

/**
 * Notification event payload published to the shared notification topic.
 */
export type NotificationEventEnvelope =
  | {
      readonly eventType: "ApplicationCompleted";
      readonly aggregateId: string;
      readonly aggregateType: "LotteryApplication";
      readonly payload: ApplicationCompletedPayload;
    }
  | {
      readonly eventType: "ApplicationFailed";
      readonly aggregateId: string;
      readonly aggregateType: "IpoStock";
      readonly payload: ApplicationFailedPayload;
    }
  | {
      readonly eventType: "ImageAuthenticationFailed";
      readonly aggregateId: string;
      readonly aggregateType: "SecuritiesAccount";
      readonly payload: ImageAuthenticationFailedPayload;
    }
  | {
      readonly eventType: "OperationErrorOccurred";
      readonly aggregateId: string;
      readonly aggregateType: "OperationLog";
      readonly payload: OperationErrorOccurredPayload;
    };

/**
 * Stored lottery application record.
 */
export interface LotteryApplicationRecord {
  readonly identifier: string;
  readonly stock: string;
  readonly securitiesAccount: string;
  readonly appliedOrder: {
    readonly shares: number;
    readonly price: number;
    readonly orderedAt: string;
  };
  readonly lotteryOutcome: null;
  readonly status: "Applied";
  readonly createdAt: string;
  readonly updatedAt: string;
}

/**
 * Supported operation log event types.
 */
export const OPERATION_LOG_EVENT_TYPES = {
  APPLY_LOTTERY: "ApplyLottery",
} as const;

/**
 * Supported operation log event types.
 */
export type OperationLogEventType =
  (typeof OPERATION_LOG_EVENT_TYPES)[keyof typeof OPERATION_LOG_EVENT_TYPES];

/**
 * Stored operation log entry.
 */
export interface OperationLogEntry {
  readonly identifier: string;
  readonly application: string | null;
  readonly eventType: OperationLogEventType;
  readonly serviceName: string;
  readonly status: "Success" | "Failure";
  readonly message: string;
  readonly errorMessage: string | null;
  readonly executedAt: string;
}
