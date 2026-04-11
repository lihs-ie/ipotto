/**
 * Result of a lottery application attempt on the broker site.
 */
export type ApplicationResult =
  | { readonly status: "success" }
  | { readonly status: "failure"; readonly reason: string }
  | { readonly status: "already_applied" }
  | { readonly status: "insufficient_balance" };

/**
 * Creates a successful application result.
 */
export function createSuccessfulApplicationResult(): ApplicationResult {
  return { status: "success" };
}

/**
 * Creates a failed application result with a reason.
 */
export function createFailedApplicationResult(
  reason: string,
): ApplicationResult {
  return { status: "failure", reason };
}

/**
 * Creates a result for an already applied stock.
 */
export function createAlreadyAppliedResult(): ApplicationResult {
  return { status: "already_applied" };
}

/**
 * Creates a result for insufficient balance.
 */
export function createInsufficientBalanceResult(): ApplicationResult {
  return { status: "insufficient_balance" };
}

/**
 * Lottery result returned by the broker service.
 */
export type LotteryResult =
  | "Won"
  | "Lost"
  | "Alternate"
  | null;
