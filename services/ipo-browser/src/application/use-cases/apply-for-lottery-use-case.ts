import type {
  ActiveSecuritiesAccount,
  AccountCredential,
} from "../../domain/account-credential.js";
import type { ApplicationResult } from "../../domain/application-result.js";
import { generateUlid } from "../../domain/generate-ulid.js";
import type { ExclusionEntry, TargetIpoStock } from "../../domain/ipo-stock.js";
import type {
  LotteryApplicationRecord,
  NotificationEventEnvelope,
  OperationLogEntry,
} from "../../domain/notification-event.js";

/**
 * Apply command input.
 */
export interface ApplyForLotteryInput {
  readonly targetDate: string;
}

/**
 * Per-attempt result entry.
 */
export interface ApplicationResultEntry {
  readonly accountId: string;
  readonly stockId: string;
  readonly result: ApplicationResult["status"];
  readonly reason: string | null;
}

/**
 * Apply command output.
 */
export interface ApplyForLotteryOutput {
  readonly appliedCount: number;
  readonly skippedCount: number;
  readonly failedCount: number;
  readonly results: readonly ApplicationResultEntry[];
}

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
      await this.eventPublisher.publish({
        eventType: "OperationErrorOccurred",
        aggregateId: generateUlid(),
        aggregateType: "OperationLog",
        payload: {
          service_name: "ipo-browser",
          operation_type: "apply_lottery",
          error_message: "No active securities account found",
          occurred_at: this.nowProvider.now().toISOString(),
        },
      });

      return {
        appliedCount: 0,
        skippedCount: 0,
        failedCount: 0,
        results: [],
      };
    }

    const [stocks, exclusions] = await Promise.all([
      this.stockRepository.findInBookBuildingPeriod(input.targetDate),
      this.exclusionRepository.findAll(),
    ]);

    const results: ApplicationResultEntry[] = [];
    let appliedCount = 0;
    let skippedCount = 0;
    let failedCount = 0;

    for (const account of accounts) {
      for (const stock of stocks) {
        const alreadyApplied =
          await this.applicationRepository.existsByStockAndAccount(
            stock.identifier,
            account.identifier,
          );

        if (!isEligible(stock, exclusions, alreadyApplied)) {
          skippedCount += 1;
          results.push({
            accountId: account.identifier,
            stockId: stock.identifier,
            result: "already_applied",
            reason: alreadyApplied ? "already applied" : "excluded",
          });
          continue;
        }

        const brokerResult = await this.brokerPort.applyForIpo(account, stock);
        const executedAt = this.nowProvider.now().toISOString();

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
            identifier: generateUlid(),
            application: applicationId,
            eventType: "ApplyLottery",
            serviceName: "ipo-browser",
            status: "Success",
            message: `${stock.companyName} のIPO抽選に申し込みました`,
            errorMessage: null,
            executedAt,
          });
          await this.eventPublisher.publish({
            eventType: "ApplicationCompleted",
            aggregateId: applicationId,
            aggregateType: "LotteryApplication",
            payload: {
              identifier: applicationId,
              stock: stock.identifier,
              securities_account: account.identifier,
              applied_shares: stock.shares,
              applied_price: stock.price,
              applied_at: executedAt,
            },
          });
          appliedCount += 1;
          results.push({
            accountId: account.identifier,
            stockId: stock.identifier,
            result: "success",
            reason: null,
          });
          continue;
        }

        const reason =
          brokerResult.status === "failure"
            ? brokerResult.reason
            : brokerResult.status === "insufficient_balance"
              ? "insufficient balance"
              : "already applied";

        if (brokerResult.status === "already_applied") {
          skippedCount += 1;
        } else {
          failedCount += 1;
        }

        await this.operationLogRepository.save({
          identifier: generateUlid(),
          application: null,
          eventType: "ApplyLottery",
          serviceName: "ipo-browser",
          status: "Failure",
          message: `${stock.companyName} のIPO抽選申し込みに失敗しました`,
          errorMessage: reason,
          executedAt,
        });
        await this.eventPublisher.publish({
          eventType: "ApplicationFailed",
          aggregateId: stock.identifier,
          aggregateType: "IpoStock",
          payload: {
            identifier: generateUlid(),
            stock: stock.identifier,
            securities_account: account.identifier,
            error_message: reason,
            failed_at: executedAt,
          },
        });
        results.push({
          accountId: account.identifier,
          stockId: stock.identifier,
          result: brokerResult.status,
          reason,
        });
      }
    }

    return {
      appliedCount,
      skippedCount,
      failedCount,
      results,
    };
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
    throw new Error("targetDate must be a valid ISO date");
  }
  if (target.getTime() > today.getTime()) {
    throw new Error("targetDate must not be in the future");
  }
}
