import type {
  ActiveSecuritiesAccount,
  AccountCredential,
} from "../../domain/account-credential.js";
import type {
  ApplicationFailureCategory,
  ApplicationResult,
} from "../../domain/application-result.js";
import { generateUlid } from "../../domain/generate-ulid.js";
import type { ExclusionEntry, TargetIpoStock } from "../../domain/ipo-stock.js";
import type {
  ApplicationCompletedPayload,
  ApplicationFailedPayload,
  ImageAuthenticationFailedPayload,
  LotteryApplicationRecord,
  NotificationEventEnvelope,
  OperationLogEntry,
  OperationErrorOccurredPayload,
} from "../../domain/notification-event.js";
import {
  OPERATION_LOG_EVENT_TYPES,
} from "../../domain/notification-event.js";

/**
 * Apply command input.
 */
export interface ApplyForLotteryInput {
  readonly targetDate: string;
}

/**
 * Validation error raised for invalid DD-101 input.
 */
export class ApplyForLotteryValidationError extends Error {
  /**
   * Creates the validation error.
   */
  public constructor(message: string) {
    super(message);
    this.name = "ApplyForLotteryValidationError";
  }
}

/**
 * Per-attempt result entry.
 */
export interface ApplicationResultEntry {
  readonly accountId: string;
  readonly stockId: string;
  readonly result: ApplicationResult["status"];
  readonly reason: string | null;
  readonly failureCategory: ApplicationFailureCategory | null;
}

/**
 * Per-account execution summary.
 */
export interface AccountApplicationSummary {
  readonly accountId: string;
  readonly appliedCount: number;
  readonly skippedCount: number;
  readonly failedCount: number;
}

/**
 * Per-stock execution summary.
 */
export interface StockApplicationSummary {
  readonly stockId: string;
  readonly appliedCount: number;
  readonly skippedCount: number;
  readonly failedCount: number;
}

/**
 * Apply command output.
 */
export interface ApplyForLotteryOutput {
  readonly appliedCount: number;
  readonly skippedCount: number;
  readonly failedCount: number;
  readonly results: readonly ApplicationResultEntry[];
  readonly accountSummaries: readonly AccountApplicationSummary[];
  readonly stockSummaries: readonly StockApplicationSummary[];
}

type FailedBrokerResult = Extract<
  ApplicationResult,
  { readonly status: "failure" | "insufficient_balance" }
>;

interface MutableSummary {
  appliedCount: number;
  skippedCount: number;
  failedCount: number;
}

type AttemptOutcome = "applied" | "skipped" | "failed";

const OPERATION_LOG_MESSAGES = {
  success: (companyName: string): string =>
    `${companyName} のIPO抽選に申し込みました`,
  alreadyApplied: (companyName: string): string =>
    `${companyName} は申込済みのためスキップしました`,
  insufficientBalance: (companyName: string): string =>
    `${companyName} は買付余力不足のため申込できませんでした`,
  imageAuthentication: (companyName: string): string =>
    `${companyName} の画像認証に失敗しました`,
  mailRetrieval: (companyName: string): string =>
    `${companyName} の認証メール取得に失敗しました`,
  applicationFailure: (companyName: string): string =>
    `${companyName} のIPO抽選申し込みに失敗しました`,
  unexpectedFailure: (companyName: string): string =>
    `${companyName} のIPO抽選処理中に予期しないエラーが発生しました`,
} as const;

const OPERATION_LOG_ERROR_MESSAGES = {
  insufficientBalance: "insufficient balance",
  mailRetrieval: "mail retrieval failed",
  imageAuthentication: "image authentication failed",
  loginFailed: "login failed",
  twoFactorPageNotReached: "2FA page not reached",
  selectorMissing: "selector missing",
  unexpectedPageTransition: "unexpected page transition",
  secretAccessFailed: "secret access failed",
  applicationFailure: "application failed",
  unexpectedWorkflowFailure: "unexpected workflow failure",
} as const;

/**
 * Repository for active broker accounts.
 */
export interface ActiveSecuritiesAccountRepository {
  /**
   * Returns active broker accounts.
   */
  findActive(): Promise<readonly ActiveSecuritiesAccount[]>;
}

/**
 * Repository for target IPO stocks.
 */
export interface IpoStockRepository {
  /**
   * Returns stocks in the given book-building date.
   */
  findInBookBuildingPeriod(targetDate: string): Promise<readonly TargetIpoStock[]>;
}

/**
 * Repository for exclusion entries.
 */
export interface ExclusionRepository {
  /**
   * Returns all exclusions.
   */
  findAll(): Promise<readonly ExclusionEntry[]>;
}

/**
 * Repository for lottery applications.
 */
export interface LotteryApplicationRepository {
  /**
   * Checks whether the stock/account pair already exists.
   */
  existsByStockAndAccount(
    stockId: string,
    accountId: string,
  ): Promise<boolean>;

  /**
   * Persists a successful lottery application.
   */
  save(record: LotteryApplicationRecord): Promise<void>;
}

/**
 * Repository for operation logs.
 */
export interface OperationLogRepository {
  /**
   * Persists an operation log entry.
   */
  save(entry: OperationLogEntry): Promise<void>;
}

/**
 * Event publisher for notification fan-out.
 */
export interface NotificationEventPublisher {
  /**
   * Publishes a notification event.
   */
  publish(event: NotificationEventEnvelope): Promise<void>;
}

/**
 * Browser integration port used by the apply workflow.
 */
export interface BrokerOperationPort {
  /**
   * Applies for an IPO stock with the given account.
   */
  applyForIpo(
    account: ActiveSecuritiesAccount,
    stock: TargetIpoStock,
  ): Promise<ApplicationResult>;

  /**
   * Tests broker connectivity.
   */
  testConnection(credential: AccountCredential): Promise<{
    readonly success: boolean;
    readonly message: string;
    readonly testedAt: string;
  }>;

  /**
   * Checks a lottery result by stock identifier.
   */
  checkLotteryResult(
    credential: AccountCredential,
    stockIdentifier: string,
  ): Promise<"Won" | "Lost" | "Alternate" | null>;
}

/**
 * Time source used by the use case.
 */
export interface NowProvider {
  /**
   * Returns the current timestamp.
   */
  now(): Date;
}

/**
 * DD-101: applies for IPO lotteries for active accounts.
 */
export class ApplyForLotteryUseCase {
  /**
   * Creates the use case.
   */
  public constructor(
    private readonly accountRepository: ActiveSecuritiesAccountRepository,
    private readonly stockRepository: IpoStockRepository,
    private readonly exclusionRepository: ExclusionRepository,
    private readonly applicationRepository: LotteryApplicationRepository,
    private readonly operationLogRepository: OperationLogRepository,
    private readonly eventPublisher: NotificationEventPublisher,
    private readonly brokerPort: BrokerOperationPort,
    private readonly nowProvider: NowProvider,
  ) {}

  /**
   * Executes DD-101.
   */
  public async execute(
    input: ApplyForLotteryInput,
  ): Promise<ApplyForLotteryOutput> {
    assertTargetDateIsNotFuture(input.targetDate, this.nowProvider.now());

    const accounts = await this.accountRepository.findActive();
    if (accounts.length === 0) {
      const occurredAt = this.nowProvider.now().toISOString();
      await this.eventPublisher.publish({
        eventType: "OperationErrorOccurred",
        aggregateId: generateUlid(),
        aggregateType: "OperationLog",
        payload: buildOperationErrorPayload(
          "No active securities account found",
          occurredAt,
        ),
      });

      return {
        appliedCount: 0,
        skippedCount: 0,
        failedCount: 0,
        results: [],
        accountSummaries: [],
        stockSummaries: [],
      };
    }

    const executableAccounts = accounts.filter(hasUsableImapCredential);
    if (executableAccounts.length === 0) {
      const occurredAt = this.nowProvider.now().toISOString();
      await this.eventPublisher.publish({
        eventType: "OperationErrorOccurred",
        aggregateId: generateUlid(),
        aggregateType: "OperationLog",
        payload: buildOperationErrorPayload(
          "No active securities account with IMAP credential found",
          occurredAt,
        ),
      });

      return {
        appliedCount: 0,
        skippedCount: 0,
        failedCount: 0,
        results: [],
        accountSummaries: [],
        stockSummaries: [],
      };
    }

    const [stocks, exclusions] = await Promise.all([
      this.stockRepository.findInBookBuildingPeriod(input.targetDate),
      this.exclusionRepository.findAll(),
    ]);

    const results: ApplicationResultEntry[] = [];
    const globalSummary = createMutableSummary();
    const accountSummaries = new Map<string, MutableSummary>(
      executableAccounts.map((account) => [account.identifier, createMutableSummary()]),
    );
    const stockSummaries = new Map<string, MutableSummary>(
      stocks.map((stock) => [stock.identifier, createMutableSummary()]),
    );

    for (const account of executableAccounts) {
      for (const stock of stocks) {
        try {
          const alreadyApplied =
            await this.applicationRepository.existsByStockAndAccount(
              stock.identifier,
              account.identifier,
            );

          if (!isEligible(stock, exclusions, alreadyApplied)) {
            recordAttemptResult(
              globalSummary,
              accountSummaries,
              stockSummaries,
              {
                accountId: account.identifier,
                stockId: stock.identifier,
                result: "already_applied",
                reason: alreadyApplied ? "already applied" : "excluded",
                failureCategory: null,
              },
              results,
            );
            continue;
          }

          const brokerResult = await this.brokerPort.applyForIpo(account, stock);
          const executedAt = captureExecutedAt(this.nowProvider);

          if (brokerResult.status === "success") {
            const applicationId = generateUlid();
            await this.applicationRepository.save({
              identifier: applicationId,
              stock: stock.identifier,
              securitiesAccount: account.identifier,
              appliedOrder: {
                shares: stock.shares,
                price: stock.price,
                orderedAt: executedAt,
              },
              lotteryOutcome: null,
              status: "Applied",
              createdAt: executedAt,
              updatedAt: executedAt,
            });
            await this.operationLogRepository.save({
              ...buildSuccessfulOperationLog(applicationId, stock, executedAt),
            });
            await this.eventPublisher.publish({
              eventType: "ApplicationCompleted",
              aggregateId: applicationId,
              aggregateType: "LotteryApplication",
              payload: buildApplicationCompletedPayload(
                applicationId,
                account,
                stock,
                executedAt,
              ),
            });
            recordAttemptResult(
              globalSummary,
              accountSummaries,
              stockSummaries,
              {
                accountId: account.identifier,
                stockId: stock.identifier,
                result: "success",
                reason: null,
                failureCategory: null,
              },
              results,
            );
            continue;
          }

          if (brokerResult.status === "already_applied") {
            await this.operationLogRepository.save({
              ...buildAlreadyAppliedOperationLog(stock, executedAt),
            });
            recordAttemptResult(
              globalSummary,
              accountSummaries,
              stockSummaries,
              {
                accountId: account.identifier,
                stockId: stock.identifier,
                result: brokerResult.status,
                reason: "already applied",
                failureCategory: null,
              },
              results,
            );
            continue;
          }

          await this.operationLogRepository.save({
            ...buildFailureOperationLog(stock, brokerResult, executedAt),
          });
          const failureEvent = buildFailureNotificationEvent(
            account,
            stock,
            brokerResult,
            executedAt,
          );
          await this.eventPublisher.publish(failureEvent);
          recordAttemptResult(
            globalSummary,
            accountSummaries,
            stockSummaries,
            {
              accountId: account.identifier,
              stockId: stock.identifier,
              result: brokerResult.status,
              reason: extractFailureReason(brokerResult),
              failureCategory:
                brokerResult.status === "failure"
                  ? brokerResult.category
                  : null,
            },
            results,
          );
        } catch (error) {
          const executedAt = captureExecutedAt(this.nowProvider);
          const reason = extractUnexpectedFailureReason(error);
          await saveOperationLogSafely(
            this.operationLogRepository,
            buildUnexpectedFailureOperationLog(stock, reason, executedAt),
          );
          recordAttemptResult(
            globalSummary,
            accountSummaries,
            stockSummaries,
            {
              accountId: account.identifier,
              stockId: stock.identifier,
              result: "failure",
              reason,
              failureCategory: "application",
            },
            results,
          );
        }
      }
    }

    return {
      appliedCount: globalSummary.appliedCount,
      skippedCount: globalSummary.skippedCount,
      failedCount: globalSummary.failedCount,
      results,
      accountSummaries: buildAccountSummaries(accountSummaries),
      stockSummaries: buildStockSummaries(stockSummaries),
    };
  }
}

/**
 * Builds the payload for an operation-level error event.
 */
function buildOperationErrorPayload(
  errorMessage: string,
  occurredAt: string,
): OperationErrorOccurredPayload {
  return {
    service_name: "ipo-browser",
    operation_type: "apply_lottery",
    error_message: errorMessage,
    occurred_at: occurredAt,
  };
}

/**
 * Captures the unified execution timestamp at the point where the broker result
 * has been determined and the workflow is about to persist/publish outcomes.
 */
function captureExecutedAt(nowProvider: NowProvider): string {
  return nowProvider.now().toISOString();
}

/**
 * Builds an operation log entry for successful applications.
 */
function buildSuccessfulOperationLog(
  applicationId: string,
  stock: TargetIpoStock,
  executedAt: string,
): OperationLogEntry {
  return {
    identifier: generateUlid(),
    application: applicationId,
    eventType: OPERATION_LOG_EVENT_TYPES.APPLY_LOTTERY,
    serviceName: "ipo-browser",
    status: "Success",
    message: OPERATION_LOG_MESSAGES.success(stock.companyName),
    errorMessage: null,
    executedAt,
  };
}

/**
 * Builds a success payload for completed applications.
 */
function buildApplicationCompletedPayload(
  applicationId: string,
  account: ActiveSecuritiesAccount,
  stock: TargetIpoStock,
  executedAt: string,
): ApplicationCompletedPayload {
  return {
    identifier: applicationId,
    stock: stock.identifier,
    securities_account: account.identifier,
    applied_shares: stock.shares,
    applied_price: stock.price,
    applied_at: executedAt,
  };
}

/**
 * Builds an operation log entry for already-applied skips.
 *
 * Skips are recorded as `Success` because the workflow intentionally decided
 * not to apply and completed without an execution error.
 */
function buildAlreadyAppliedOperationLog(
  stock: TargetIpoStock,
  executedAt: string,
): OperationLogEntry {
  return {
    identifier: generateUlid(),
    application: null,
    eventType: OPERATION_LOG_EVENT_TYPES.APPLY_LOTTERY,
    serviceName: "ipo-browser",
    status: "Success",
    message: OPERATION_LOG_MESSAGES.alreadyApplied(stock.companyName),
    errorMessage: null,
    executedAt,
  };
}

/**
 * Builds an operation log entry for failed application attempts.
 */
function buildFailureOperationLog(
  stock: TargetIpoStock,
  brokerResult: FailedBrokerResult,
  executedAt: string,
): OperationLogEntry {
  return {
    identifier: generateUlid(),
    application: null,
    eventType: OPERATION_LOG_EVENT_TYPES.APPLY_LOTTERY,
    serviceName: "ipo-browser",
    status: "Failure",
    message: buildFailureOperationMessage(stock.companyName, brokerResult),
    errorMessage: buildFailureOperationErrorMessage(brokerResult),
    executedAt,
  };
}

/**
 * Builds the correct notification event for a failed application attempt.
 */
function buildFailureNotificationEvent(
  account: ActiveSecuritiesAccount,
  stock: TargetIpoStock,
  brokerResult: FailedBrokerResult,
  executedAt: string,
): NotificationEventEnvelope {
  const identifier = generateUlid();
  if (
    brokerResult.status === "failure" &&
    (brokerResult.category === "image_authentication" ||
      brokerResult.category === "mail_retrieval")
  ) {
    const payload: ImageAuthenticationFailedPayload = {
      securities_account: account.identifier,
      failure_reason: buildExternalFailureMessage(brokerResult),
      attempt_count: 1,
      occurred_at: executedAt,
    };

    return {
      eventType: "ImageAuthenticationFailed",
      aggregateId: account.identifier,
      aggregateType: "SecuritiesAccount",
      payload,
    };
  }

  const payload: ApplicationFailedPayload = {
    identifier,
    stock: stock.identifier,
    securities_account: account.identifier,
    error_message: buildExternalFailureMessage(brokerResult),
    failed_at: executedAt,
  };

  return {
    eventType: "ApplicationFailed",
    aggregateId: stock.identifier,
    aggregateType: "IpoStock",
    payload,
  };
}

/**
 * Returns the user-facing reason for a failed broker result.
 */
function extractFailureReason(
  brokerResult: FailedBrokerResult,
): string {
  return brokerResult.status === "failure"
    ? brokerResult.reason
    : "insufficient balance";
}

/**
 * Builds an operation log message that matches the failure category.
 */
function buildFailureOperationMessage(
  companyName: string,
  brokerResult: FailedBrokerResult,
): string {
  if (brokerResult.status === "insufficient_balance") {
    return OPERATION_LOG_MESSAGES.insufficientBalance(companyName);
  }

  if (brokerResult.category === "mail_retrieval") {
    return OPERATION_LOG_MESSAGES.mailRetrieval(companyName);
  }

  if (brokerResult.category === "image_authentication") {
    return OPERATION_LOG_MESSAGES.imageAuthentication(companyName);
  }

  return OPERATION_LOG_MESSAGES.applicationFailure(companyName);
}

/**
 * Builds the external notification message for failed attempts.
 */
function buildExternalFailureMessage(
  brokerResult: FailedBrokerResult,
): string {
  if (brokerResult.status === "insufficient_balance") {
    return "買付余力が不足しているため、申し込みできませんでした";
  }

  if (brokerResult.category === "mail_retrieval") {
    return "認証メールを取得できなかったため、画像認証に失敗しました";
  }

  if (brokerResult.category === "image_authentication") {
    return "画像認証に失敗しました";
  }

  return "IPO抽選の申し込みに失敗しました";
}

/**
 * Builds a sanitized internal error message for operation logs.
 */
function buildFailureOperationErrorMessage(
  brokerResult: FailedBrokerResult,
): string {
  if (brokerResult.status === "insufficient_balance") {
    return OPERATION_LOG_ERROR_MESSAGES.insufficientBalance;
  }

  if (brokerResult.category === "mail_retrieval") {
    return OPERATION_LOG_ERROR_MESSAGES.mailRetrieval;
  }

  if (brokerResult.category === "image_authentication") {
    return OPERATION_LOG_ERROR_MESSAGES.imageAuthentication;
  }

  return sanitizeApplicationFailureReason(brokerResult.reason);
}

/**
 * Creates a mutable summary used for global, account, and stock counters.
 */
function createMutableSummary(): MutableSummary {
  return {
    appliedCount: 0,
    skippedCount: 0,
    failedCount: 0,
  };
}

/**
 * Records one attempt result and updates all counters from the same rules.
 */
function recordAttemptResult(
  globalSummary: MutableSummary,
  accountSummaries: ReadonlyMap<string, MutableSummary>,
  stockSummaries: ReadonlyMap<string, MutableSummary>,
  result: ApplicationResultEntry,
  results: ApplicationResultEntry[],
): void {
  const outcome = classifyAttemptOutcome(result);
  incrementSummary(globalSummary, outcome);
  incrementSummary(accountSummaries.get(result.accountId), outcome);
  incrementSummary(stockSummaries.get(result.stockId), outcome);
  results.push(result);
}

/**
 * Classifies one result entry into the fixed count buckets.
 *
 * `already_applied` is treated as a skip, not a failure. The workflow completed
 * successfully and intentionally took no broker action for that pair.
 */
function classifyAttemptOutcome(
  result: ApplicationResultEntry,
): AttemptOutcome {
  if (result.result === "success") {
    return "applied";
  }

  if (result.result === "already_applied") {
    return "skipped";
  }

  return "failed";
}

/**
 * Increments the summary for the given outcome.
 */
function incrementSummary(
  summary: MutableSummary | undefined,
  outcome: AttemptOutcome,
): void {
  if (summary === undefined) {
    return;
  }

  if (outcome === "applied") {
    summary.appliedCount += 1;
    return;
  }

  if (outcome === "skipped") {
    summary.skippedCount += 1;
    return;
  }

  summary.failedCount += 1;
}

/**
 * Builds immutable per-account summaries from the mutable map.
 */
function buildAccountSummaries(
  summaries: ReadonlyMap<string, MutableSummary>,
): readonly AccountApplicationSummary[] {
  return [...summaries.entries()].map(([accountId, summary]) => ({
    accountId,
    appliedCount: summary.appliedCount,
    skippedCount: summary.skippedCount,
    failedCount: summary.failedCount,
  }));
}

/**
 * Builds immutable per-stock summaries from the mutable map.
 */
function buildStockSummaries(
  summaries: ReadonlyMap<string, MutableSummary>,
): readonly StockApplicationSummary[] {
  return [...summaries.entries()].map(([stockId, summary]) => ({
    stockId,
    appliedCount: summary.appliedCount,
    skippedCount: summary.skippedCount,
    failedCount: summary.failedCount,
  }));
}

/**
 * Returns a safe reason string for unexpected workflow failures.
 */
function extractUnexpectedFailureReason(error: unknown): string {
  return error instanceof Error ? error.message : "unexpected workflow failure";
}

/**
 * Reduces raw application errors into safe operation-log messages.
 */
function sanitizeApplicationFailureReason(reason: string): string {
  const normalizedReason = reason.trim().toLowerCase();

  if (
    normalizedReason.includes("invalid login") ||
    normalizedReason.includes("login failed")
  ) {
    return OPERATION_LOG_ERROR_MESSAGES.loginFailed;
  }

  if (normalizedReason.includes("2fa page not reached")) {
    return OPERATION_LOG_ERROR_MESSAGES.twoFactorPageNotReached;
  }

  if (normalizedReason.includes("selector missing")) {
    return OPERATION_LOG_ERROR_MESSAGES.selectorMissing;
  }

  if (normalizedReason.includes("unexpected page transition")) {
    return OPERATION_LOG_ERROR_MESSAGES.unexpectedPageTransition;
  }

  if (
    normalizedReason.includes("secret manager") ||
    normalizedReason.includes("loginpassword=") ||
    normalizedReason.includes("mailpassword=") ||
    normalizedReason.includes("credential")
  ) {
    return OPERATION_LOG_ERROR_MESSAGES.secretAccessFailed;
  }

  return OPERATION_LOG_ERROR_MESSAGES.applicationFailure;
}

/**
 * Builds an operation log entry for unexpected workflow failures.
 */
function buildUnexpectedFailureOperationLog(
  stock: TargetIpoStock,
  reason: string,
  executedAt: string,
): OperationLogEntry {
  return {
    identifier: generateUlid(),
    application: null,
    eventType: OPERATION_LOG_EVENT_TYPES.APPLY_LOTTERY,
    serviceName: "ipo-browser",
    status: "Failure",
    message: OPERATION_LOG_MESSAGES.unexpectedFailure(stock.companyName),
    errorMessage: sanitizeApplicationFailureReason(reason),
    executedAt,
  };
}

/**
 * Saves an operation log entry without aborting the whole batch when logging fails.
 */
async function saveOperationLogSafely(
  repository: OperationLogRepository,
  entry: OperationLogEntry,
): Promise<void> {
  try {
    await repository.save(entry);
  } catch {
    // Keep the batch running even when operation log persistence fails.
  }
}

/**
 * Returns whether the stock is eligible for automatic application.
 */
function isEligible(
  stock: TargetIpoStock,
  exclusions: readonly ExclusionEntry[],
  alreadyApplied: boolean,
): boolean {
  if (alreadyApplied) {
    return false;
  }

  return !exclusions.some(
    (entry) => entry.companyName.trim() === stock.companyName.trim(),
  );
}

/**
 * Validates that the target date is not in the future.
 */
function assertTargetDateIsNotFuture(targetDate: string, now: Date): void {
  const target = new Date(`${targetDate}T00:00:00.000Z`);
  const today = new Date(now.toISOString().slice(0, 10) + "T00:00:00.000Z");
  if (Number.isNaN(target.getTime())) {
    throw new ApplyForLotteryValidationError("targetDate must be a valid ISO date");
  }
  if (target.getTime() > today.getTime()) {
    throw new ApplyForLotteryValidationError("targetDate must not be in the future");
  }
}

/**
 * Returns whether the account has enough IMAP credential data to fetch authentication mails.
 */
function hasUsableImapCredential(account: ActiveSecuritiesAccount): boolean {
  const mailCredential = account.credential.mailCredential;
  return (
    mailCredential.mailAddress.trim() !== "" &&
    mailCredential.mailPassword.trim() !== "" &&
    mailCredential.imapHost.trim() !== "" &&
    mailCredential.imapPort > 0
  );
}
