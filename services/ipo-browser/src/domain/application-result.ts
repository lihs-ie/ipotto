/**
 * Result of a lottery application attempt on the broker site.
 */
export type ApplicationFailureCategory =
  /**
   * Browser-side application failures such as login failure, 2FA page transition
   * failure, selector mismatch, and unexpected page navigation.
   */
  | "application"
  | "image_authentication"
  | "mail_retrieval";

/**
 * Result of a lottery application attempt on the broker site.
 */
export type ApplicationResult =
  | { readonly status: "success" }
  | {
      readonly status: "failure";
      readonly reason: string;
      readonly category: ApplicationFailureCategory;
    }
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
  category: ApplicationFailureCategory = "application",
): ApplicationResult {
  return { status: "failure", reason, category };
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
