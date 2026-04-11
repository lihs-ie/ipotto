/**
 * Notification event payload published to the shared notification topic.
 */
export interface NotificationEventEnvelope {
  readonly eventType:
    | "ApplicationCompleted"
    | "ApplicationFailed"
    | "ImageAuthenticationFailed"
    | "OperationErrorOccurred";
  readonly aggregateId: string;
  readonly aggregateType: string;
  readonly payload: Record<string, unknown>;
}

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
 * Stored operation log entry.
 */
export interface OperationLogEntry {
  readonly identifier: string;
  readonly application: string | null;
  readonly eventType: string;
  readonly serviceName: string;
  readonly status: "Success" | "Failure";
  readonly message: string;
  readonly errorMessage: string | null;
  readonly executedAt: string;
}
