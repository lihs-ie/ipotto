import { describe, expect, it } from "vitest";

import type { ActiveSecuritiesAccount } from "../../domain/account-credential.js";
import type { ApplicationResult } from "../../domain/application-result.js";
import type { ExclusionEntry, TargetIpoStock } from "../../domain/ipo-stock.js";
import type {
  LotteryApplicationRecord,
  NotificationEventEnvelope,
  OperationLogEntry,
} from "../../domain/notification-event.js";
import { OPERATION_LOG_EVENT_TYPES } from "../../domain/notification-event.js";
import {
  ApplyForLotteryUseCase,
  type ActiveSecuritiesAccountRepository,
  type BrokerOperationPort,
  type ExclusionRepository,
  type IpoStockRepository,
  type LotteryApplicationRepository,
  type NotificationEventPublisher,
  type OperationLogRepository,
} from "./apply-for-lottery-use-case.js";

class InMemoryAccountRepository implements ActiveSecuritiesAccountRepository {
  public constructor(private readonly accounts: readonly ActiveSecuritiesAccount[]) {}

  public async findActive(): Promise<readonly ActiveSecuritiesAccount[]> {
    return this.accounts;
  }
}

class InMemoryStockRepository implements IpoStockRepository {
  public constructor(private readonly stocks: readonly TargetIpoStock[]) {}

  public async findInBookBuildingPeriod(): Promise<readonly TargetIpoStock[]> {
    return this.stocks;
  }
}

class InMemoryExclusionRepository implements ExclusionRepository {
  public constructor(private readonly exclusions: readonly ExclusionEntry[]) {}

  public async findAll(): Promise<readonly ExclusionEntry[]> {
    return this.exclusions;
  }
}

class InMemoryApplicationRepository implements LotteryApplicationRepository {
  public readonly records: LotteryApplicationRecord[] = [];
  private readonly alreadyAppliedPairs: Set<string>;

  public constructor(
    alreadyAppliedPairs: readonly string[] = [],
  ) {
    this.alreadyAppliedPairs = new Set(alreadyAppliedPairs);
  }

  public async existsByStockAndAccount(
    stockId: string,
    accountId: string,
  ): Promise<boolean> {
    return this.alreadyAppliedPairs.has(`${stockId}:${accountId}`);
  }

  public async save(record: LotteryApplicationRecord): Promise<void> {
    this.records.push(record);
  }
}

class InMemoryOperationLogRepository implements OperationLogRepository {
  public readonly entries: OperationLogEntry[] = [];

  public async save(entry: OperationLogEntry): Promise<void> {
    this.entries.push(entry);
  }
}

class InMemoryNotificationPublisher implements NotificationEventPublisher {
  public readonly events: NotificationEventEnvelope[] = [];

  public async publish(event: NotificationEventEnvelope): Promise<void> {
    this.events.push(event);
  }
}

class SequenceNowProvider {
  private index = 0;

  public constructor(private readonly timestamps: readonly string[]) {}

  public now(): Date {
    const timestamp =
      this.timestamps[this.index] ??
      this.timestamps[this.timestamps.length - 1] ??
      "2026-04-10T00:00:00.000Z";
    this.index += 1;
    return new Date(timestamp);
  }
}

class SuccessfulBrokerPort implements BrokerOperationPort {
  public async applyForIpo() {
    return { status: "success" } as const;
  }

  public async testConnection() {
    return {
      success: true,
      message: "ok",
      testedAt: "2026-04-11T00:00:00.000Z",
    } as const;
  }

  public async checkLotteryResult() {
    return null;
  }
}

class MailRetrievalFailureBrokerPort implements BrokerOperationPort {
  public async applyForIpo() {
    return {
      status: "failure",
      reason:
        "imap auth failed for test@example.com with password=mail-password within 120000ms",
      category: "mail_retrieval",
    } as const;
  }

  public async testConnection() {
    return {
      success: true,
      message: "ok",
      testedAt: "2026-04-11T00:00:00.000Z",
    } as const;
  }

  public async checkLotteryResult() {
    return null;
  }
}

class ImageAuthenticationFailureBrokerPort implements BrokerOperationPort {
  public async applyForIpo() {
    return {
      status: "failure",
      reason: "image selection did not match the requested keywords",
      category: "image_authentication",
    } as const;
  }

  public async testConnection() {
    return {
      success: true,
      message: "ok",
      testedAt: "2026-04-11T00:00:00.000Z",
    } as const;
  }

  public async checkLotteryResult() {
    return null;
  }
}

class ApplicationFailureBrokerPort implements BrokerOperationPort {
  public async applyForIpo() {
    return {
      status: "failure",
      reason:
        "selector missing: #passwordInputText on https://rakuten.example.test/apply",
      category: "application",
    } as const;
  }

  public async testConnection() {
    return {
      success: true,
      message: "ok",
      testedAt: "2026-04-11T00:00:00.000Z",
    } as const;
  }

  public async checkLotteryResult() {
    return null;
  }
}

class SecretFailureBrokerPort implements BrokerOperationPort {
  public async applyForIpo() {
    return {
      status: "failure",
      reason:
        "secret manager returned loginPassword=super-secret-value for account-1",
      category: "application",
    } as const;
  }

  public async testConnection() {
    return {
      success: true,
      message: "ok",
      testedAt: "2026-04-11T00:00:00.000Z",
    } as const;
  }

  public async checkLotteryResult() {
    return null;
  }
}

class UnexpectedFailureBrokerPort implements BrokerOperationPort {
  public async applyForIpo(): Promise<ApplicationResult> {
    throw new Error(
      "selector missing: #passwordInputText on https://rakuten.example.test/apply?token=secret",
    );
  }

  public async testConnection() {
    return {
      success: true,
      message: "ok",
      testedAt: "2026-04-11T00:00:00.000Z",
    } as const;
  }

  public async checkLotteryResult() {
    return null;
  }
}

class AlreadyAppliedBrokerPort implements BrokerOperationPort {
  public async applyForIpo() {
    return { status: "already_applied" } as const;
  }

  public async testConnection() {
    return {
      success: true,
      message: "ok",
      testedAt: "2026-04-11T00:00:00.000Z",
    } as const;
  }

  public async checkLotteryResult() {
    return null;
  }
}

class InsufficientBalanceBrokerPort implements BrokerOperationPort {
  public async applyForIpo() {
    return { status: "insufficient_balance" } as const;
  }

  public async testConnection() {
    return {
      success: true,
      message: "ok",
      testedAt: "2026-04-11T00:00:00.000Z",
    } as const;
  }

  public async checkLotteryResult() {
    return null;
  }
}

class SelectiveBrokerPort implements BrokerOperationPort {
  public async applyForIpo(
    activeAccount: ActiveSecuritiesAccount,
    targetStock: TargetIpoStock,
  ) {
    if (
      activeAccount.identifier === "account-1" &&
      targetStock.identifier === "stock-1"
    ) {
      return { status: "success" } as const;
    }

    if (
      activeAccount.identifier === "account-1" &&
      targetStock.identifier === "stock-2"
    ) {
      throw new Error("selector missing");
    }

    return { status: "insufficient_balance" } as const;
  }

  public async testConnection() {
    return {
      success: true,
      message: "ok",
      testedAt: "2026-04-11T00:00:00.000Z",
    } as const;
  }

  public async checkLotteryResult() {
    return null;
  }
}

const account: ActiveSecuritiesAccount = {
  identifier: "account-1",
  securitiesCompany: "Rakuten",
  credential: {
    loginId: "login",
    loginPassword: "password",
    tradingPassword: "1234",
    mailCredential: {
      mailAddress: "test@example.com",
      mailPassword: "mail-password",
      imapHost: "imap.example.com",
      imapPort: 993,
    },
  },
};

const stock: TargetIpoStock = {
  identifier: "stock-1",
  companyName: "テスト株式会社",
  price: 1400,
  shares: 100,
  bookBuildingStartDate: "2026-04-01",
  bookBuildingEndDate: "2026-04-10",
};

const secondAccount: ActiveSecuritiesAccount = {
  ...account,
  identifier: "account-2",
  credential: {
    ...account.credential,
    loginId: "login-2",
  },
};

const missingImapCredentialAccount: ActiveSecuritiesAccount = {
  ...account,
  identifier: "account-missing-imap",
  credential: {
    ...account.credential,
    mailCredential: {
      ...account.credential.mailCredential,
      mailPassword: "",
    },
  },
};

const secondStock: TargetIpoStock = {
  ...stock,
  identifier: "stock-2",
  companyName: "第二テスト株式会社",
};

describe("ApplyForLotteryUseCase", () => {
  it("publishes OperationErrorOccurred only when no active account exists", async () => {
    const applicationRepository = new InMemoryApplicationRepository();
    const operationLogRepository = new InMemoryOperationLogRepository();
    const eventPublisher = new InMemoryNotificationPublisher();
    const useCase = new ApplyForLotteryUseCase(
      new InMemoryAccountRepository([]),
      new InMemoryStockRepository([stock]),
      new InMemoryExclusionRepository([]),
      applicationRepository,
      operationLogRepository,
      eventPublisher,
      new SuccessfulBrokerPort(),
      { now: () => new Date("2026-04-10T00:00:00.000Z") },
    );

    const output = await useCase.execute({ targetDate: "2026-04-10" });

    expect(output).toEqual({
      appliedCount: 0,
      skippedCount: 0,
      failedCount: 0,
      results: [],
      accountSummaries: [],
      stockSummaries: [],
    });
    expect(eventPublisher.events).toHaveLength(1);
    expect(eventPublisher.events[0]).toMatchObject({
      eventType: "OperationErrorOccurred",
      aggregateType: "OperationLog",
      payload: {
        service_name: "ipo-browser",
        operation_type: "apply_lottery",
        error_message: "No active securities account found",
      },
    });
    expect(operationLogRepository.entries).toHaveLength(0);
  });

  it("publishes OperationErrorOccurred when no active account has a usable IMAP credential", async () => {
    const applicationRepository = new InMemoryApplicationRepository();
    const operationLogRepository = new InMemoryOperationLogRepository();
    const eventPublisher = new InMemoryNotificationPublisher();
    const useCase = new ApplyForLotteryUseCase(
      new InMemoryAccountRepository([missingImapCredentialAccount]),
      new InMemoryStockRepository([stock]),
      new InMemoryExclusionRepository([]),
      applicationRepository,
      operationLogRepository,
      eventPublisher,
      new SuccessfulBrokerPort(),
      { now: () => new Date("2026-04-10T00:00:00.000Z") },
    );

    const output = await useCase.execute({ targetDate: "2026-04-10" });

    expect(output).toEqual({
      appliedCount: 0,
      skippedCount: 0,
      failedCount: 0,
      results: [],
      accountSummaries: [],
      stockSummaries: [],
    });
    expect(eventPublisher.events[0]).toMatchObject({
      eventType: "OperationErrorOccurred",
      payload: {
        error_message: "No active securities account with IMAP credential found",
      },
    });
  });

  it("excludes accounts with missing IMAP credentials from the apply target", async () => {
    const applicationRepository = new InMemoryApplicationRepository();
    const operationLogRepository = new InMemoryOperationLogRepository();
    const eventPublisher = new InMemoryNotificationPublisher();
    const useCase = new ApplyForLotteryUseCase(
      new InMemoryAccountRepository([missingImapCredentialAccount, account]),
      new InMemoryStockRepository([stock]),
      new InMemoryExclusionRepository([]),
      applicationRepository,
      operationLogRepository,
      eventPublisher,
      new SuccessfulBrokerPort(),
      { now: () => new Date("2026-04-10T00:00:00.000Z") },
    );

    const output = await useCase.execute({ targetDate: "2026-04-10" });

    expect(output.accountSummaries).toEqual([
      {
        accountId: "account-1",
        appliedCount: 1,
        skippedCount: 0,
        failedCount: 0,
      },
    ]);
    expect(output.results).toHaveLength(1);
    expect(output.results[0]).toMatchObject({
      accountId: "account-1",
      stockId: "stock-1",
      result: "success",
    });
  });

  it("saves applications and publishes ApplicationCompleted", async () => {
    const applicationRepository = new InMemoryApplicationRepository();
    const operationLogRepository = new InMemoryOperationLogRepository();
    const eventPublisher = new InMemoryNotificationPublisher();
    const useCase = new ApplyForLotteryUseCase(
      new InMemoryAccountRepository([account]),
      new InMemoryStockRepository([stock]),
      new InMemoryExclusionRepository([]),
      applicationRepository,
      operationLogRepository,
      eventPublisher,
      new SuccessfulBrokerPort(),
      { now: () => new Date("2026-04-10T00:00:00.000Z") },
    );

    const output = await useCase.execute({ targetDate: "2026-04-10" });

    expect(output.appliedCount).toBe(1);
    expect(output.accountSummaries).toEqual([
      {
        accountId: "account-1",
        appliedCount: 1,
        skippedCount: 0,
        failedCount: 0,
      },
    ]);
    expect(output.stockSummaries).toEqual([
      {
        stockId: "stock-1",
        appliedCount: 1,
        skippedCount: 0,
        failedCount: 0,
      },
    ]);
    expect(applicationRepository.records).toHaveLength(1);
    expect(eventPublisher.events[0]?.eventType).toBe("ApplicationCompleted");
    expect(eventPublisher.events[0]?.payload).toMatchObject({
      stock: "stock-1",
      securities_account: "account-1",
      applied_shares: 100,
      applied_price: 1400,
    });
    expect(operationLogRepository.entries[0]).toMatchObject({
      eventType: OPERATION_LOG_EVENT_TYPES.APPLY_LOTTERY,
      application: applicationRepository.records[0]?.identifier,
      status: "Success",
      message: "テスト株式会社 のIPO抽選に申し込みました",
      errorMessage: null,
    });
    expect(output.results[0]?.failureCategory).toBeNull();
  });

  it("uses the result-determination timestamp for successful application records, logs, and events", async () => {
    const applicationRepository = new InMemoryApplicationRepository();
    const operationLogRepository = new InMemoryOperationLogRepository();
    const eventPublisher = new InMemoryNotificationPublisher();
    const nowProvider = new SequenceNowProvider([
      "2026-04-10T00:00:00.000Z",
      "2026-04-10T00:00:05.000Z",
    ]);
    const useCase = new ApplyForLotteryUseCase(
      new InMemoryAccountRepository([account]),
      new InMemoryStockRepository([stock]),
      new InMemoryExclusionRepository([]),
      applicationRepository,
      operationLogRepository,
      eventPublisher,
      new SuccessfulBrokerPort(),
      nowProvider,
    );

    await useCase.execute({ targetDate: "2026-04-10" });

    expect(applicationRepository.records[0]).toMatchObject({
      appliedOrder: { orderedAt: "2026-04-10T00:00:05.000Z" },
      createdAt: "2026-04-10T00:00:05.000Z",
      updatedAt: "2026-04-10T00:00:05.000Z",
    });
    expect(operationLogRepository.entries[0]).toMatchObject({
      executedAt: "2026-04-10T00:00:05.000Z",
    });
    expect(eventPublisher.events[0]).toMatchObject({
      payload: {
        applied_at: "2026-04-10T00:00:05.000Z",
      },
    });
  });

  it("skips excluded stocks", async () => {
    const applicationRepository = new InMemoryApplicationRepository();
    const operationLogRepository = new InMemoryOperationLogRepository();
    const eventPublisher = new InMemoryNotificationPublisher();
    const useCase = new ApplyForLotteryUseCase(
      new InMemoryAccountRepository([account]),
      new InMemoryStockRepository([stock]),
      new InMemoryExclusionRepository([
        {
          identifier: "exclusion-1",
          companyName: "テスト株式会社",
          reason: "manual",
        },
      ]),
      applicationRepository,
      operationLogRepository,
      eventPublisher,
      new SuccessfulBrokerPort(),
      { now: () => new Date("2026-04-10T00:00:00.000Z") },
    );

    const output = await useCase.execute({ targetDate: "2026-04-10" });

    expect(output.appliedCount).toBe(0);
    expect(output.skippedCount).toBe(1);
    expect(output.accountSummaries[0]).toEqual({
      accountId: "account-1",
      appliedCount: 0,
      skippedCount: 1,
      failedCount: 0,
    });
    expect(output.stockSummaries[0]).toEqual({
      stockId: "stock-1",
      appliedCount: 0,
      skippedCount: 1,
      failedCount: 0,
    });
    expect(applicationRepository.records).toHaveLength(0);
    expect(output.results[0]?.failureCategory).toBeNull();
  });

  it("publishes ImageAuthenticationFailed for mail retrieval failures", async () => {
    const applicationRepository = new InMemoryApplicationRepository();
    const operationLogRepository = new InMemoryOperationLogRepository();
    const eventPublisher = new InMemoryNotificationPublisher();
    const useCase = new ApplyForLotteryUseCase(
      new InMemoryAccountRepository([account]),
      new InMemoryStockRepository([stock]),
      new InMemoryExclusionRepository([]),
      applicationRepository,
      operationLogRepository,
      eventPublisher,
      new MailRetrievalFailureBrokerPort(),
      { now: () => new Date("2026-04-10T00:00:00.000Z") },
    );

    const output = await useCase.execute({ targetDate: "2026-04-10" });

    expect(output.failedCount).toBe(1);
    expect(output.results[0]?.failureCategory).toBe("mail_retrieval");
    expect(output.accountSummaries[0]).toEqual({
      accountId: "account-1",
      appliedCount: 0,
      skippedCount: 0,
      failedCount: 1,
    });
    expect(eventPublisher.events[0]?.eventType).toBe("ImageAuthenticationFailed");
    expect(eventPublisher.events[0]?.aggregateType).toBe("SecuritiesAccount");
    expect(eventPublisher.events[0]?.payload).toMatchObject({
      securities_account: "account-1",
      failure_reason: "認証メールを取得できなかったため、画像認証に失敗しました",
      attempt_count: 1,
    });
    expect(eventPublisher.events[0]?.payload).not.toHaveProperty("message");
    expect(JSON.stringify(eventPublisher.events[0]?.payload)).not.toContain(
      "mail-password",
    );
    expect(JSON.stringify(eventPublisher.events[0]?.payload)).not.toContain(
      "test@example.com",
    );
    expect(operationLogRepository.entries[0]).toMatchObject({
      eventType: OPERATION_LOG_EVENT_TYPES.APPLY_LOTTERY,
      application: null,
      status: "Failure",
      message: "テスト株式会社 の認証メール取得に失敗しました",
      errorMessage: "mail retrieval failed",
    });
    expect(operationLogRepository.entries[0]?.errorMessage).not.toContain(
      "mail-password",
    );
    expect(operationLogRepository.entries[0]?.errorMessage).not.toContain(
      "test@example.com",
    );
  });

  it("publishes ImageAuthenticationFailed with a fixed external message for image authentication failures", async () => {
    const applicationRepository = new InMemoryApplicationRepository();
    const operationLogRepository = new InMemoryOperationLogRepository();
    const eventPublisher = new InMemoryNotificationPublisher();
    const useCase = new ApplyForLotteryUseCase(
      new InMemoryAccountRepository([account]),
      new InMemoryStockRepository([stock]),
      new InMemoryExclusionRepository([]),
      applicationRepository,
      operationLogRepository,
      eventPublisher,
      new ImageAuthenticationFailureBrokerPort(),
      { now: () => new Date("2026-04-10T00:00:00.000Z") },
    );

    const output = await useCase.execute({ targetDate: "2026-04-10" });

    expect(output.failedCount).toBe(1);
    expect(output.results[0]?.failureCategory).toBe("image_authentication");
    expect(eventPublisher.events[0]).toMatchObject({
      eventType: "ImageAuthenticationFailed",
      aggregateType: "SecuritiesAccount",
      payload: {
        securities_account: "account-1",
        failure_reason: "画像認証に失敗しました",
        attempt_count: 1,
      },
    });
    expect(eventPublisher.events[0]?.payload).not.toHaveProperty("message");
    expect(operationLogRepository.entries[0]).toMatchObject({
      application: null,
      status: "Failure",
      message: "テスト株式会社 の画像認証に失敗しました",
      errorMessage: "image authentication failed",
    });
  });

  it("does not publish a failure event when the broker reports already applied", async () => {
    const applicationRepository = new InMemoryApplicationRepository();
    const operationLogRepository = new InMemoryOperationLogRepository();
    const eventPublisher = new InMemoryNotificationPublisher();
    const useCase = new ApplyForLotteryUseCase(
      new InMemoryAccountRepository([account]),
      new InMemoryStockRepository([stock]),
      new InMemoryExclusionRepository([]),
      applicationRepository,
      operationLogRepository,
      eventPublisher,
      new AlreadyAppliedBrokerPort(),
      { now: () => new Date("2026-04-10T00:00:00.000Z") },
    );

    const output = await useCase.execute({ targetDate: "2026-04-10" });

    expect(output.skippedCount).toBe(1);
    expect(output.failedCount).toBe(0);
    expect(output.accountSummaries[0]).toEqual({
      accountId: "account-1",
      appliedCount: 0,
      skippedCount: 1,
      failedCount: 0,
    });
    expect(eventPublisher.events).toHaveLength(0);
    expect(output.results[0]?.result).toBe("already_applied");
    expect(output.results[0]?.failureCategory).toBeNull();
    expect(operationLogRepository.entries[0]).toMatchObject({
      eventType: OPERATION_LOG_EVENT_TYPES.APPLY_LOTTERY,
      application: null,
      status: "Success",
      message: "テスト株式会社 は申込済みのためスキップしました",
      errorMessage: null,
    });
  });

  it("uses the result-determination timestamp for already-applied skip logs", async () => {
    const applicationRepository = new InMemoryApplicationRepository();
    const operationLogRepository = new InMemoryOperationLogRepository();
    const eventPublisher = new InMemoryNotificationPublisher();
    const nowProvider = new SequenceNowProvider([
      "2026-04-10T00:00:00.000Z",
      "2026-04-10T00:00:07.000Z",
    ]);
    const useCase = new ApplyForLotteryUseCase(
      new InMemoryAccountRepository([account]),
      new InMemoryStockRepository([stock]),
      new InMemoryExclusionRepository([]),
      applicationRepository,
      operationLogRepository,
      eventPublisher,
      new AlreadyAppliedBrokerPort(),
      nowProvider,
    );

    await useCase.execute({ targetDate: "2026-04-10" });

    expect(operationLogRepository.entries[0]).toMatchObject({
      executedAt: "2026-04-10T00:00:07.000Z",
    });
  });

  it("publishes ApplicationFailed with a fixed payload for insufficient balance", async () => {
    const applicationRepository = new InMemoryApplicationRepository();
    const operationLogRepository = new InMemoryOperationLogRepository();
    const eventPublisher = new InMemoryNotificationPublisher();
    const useCase = new ApplyForLotteryUseCase(
      new InMemoryAccountRepository([account]),
      new InMemoryStockRepository([stock]),
      new InMemoryExclusionRepository([]),
      applicationRepository,
      operationLogRepository,
      eventPublisher,
      new InsufficientBalanceBrokerPort(),
      { now: () => new Date("2026-04-10T00:00:00.000Z") },
    );

    const output = await useCase.execute({ targetDate: "2026-04-10" });

    expect(output.failedCount).toBe(1);
    expect(output.results[0]).toMatchObject({
      result: "insufficient_balance",
      reason: "insufficient balance",
      failureCategory: null,
    });
    expect(eventPublisher.events[0]).toMatchObject({
      eventType: "ApplicationFailed",
      aggregateId: "stock-1",
      aggregateType: "IpoStock",
      payload: {
        stock: "stock-1",
        securities_account: "account-1",
        error_message: "買付余力が不足しているため、申し込みできませんでした",
      },
    });
    expect(operationLogRepository.entries[0]).toMatchObject({
      application: null,
      status: "Failure",
      message: "テスト株式会社 は買付余力不足のため申込できませんでした",
      errorMessage: "insufficient balance",
    });
  });

  it("publishes ApplicationFailed with a sanitized external message for generic application failures", async () => {
    const applicationRepository = new InMemoryApplicationRepository();
    const operationLogRepository = new InMemoryOperationLogRepository();
    const eventPublisher = new InMemoryNotificationPublisher();
    const useCase = new ApplyForLotteryUseCase(
      new InMemoryAccountRepository([account]),
      new InMemoryStockRepository([stock]),
      new InMemoryExclusionRepository([]),
      applicationRepository,
      operationLogRepository,
      eventPublisher,
      new ApplicationFailureBrokerPort(),
      { now: () => new Date("2026-04-10T00:00:00.000Z") },
    );

    const output = await useCase.execute({ targetDate: "2026-04-10" });

    expect(output.failedCount).toBe(1);
    expect(output.results[0]).toMatchObject({
      result: "failure",
      reason:
        "selector missing: #passwordInputText on https://rakuten.example.test/apply",
      failureCategory: "application",
    });
    expect(eventPublisher.events[0]).toMatchObject({
      eventType: "ApplicationFailed",
      aggregateType: "IpoStock",
      payload: {
        stock: "stock-1",
        securities_account: "account-1",
        error_message: "IPO抽選の申し込みに失敗しました",
      },
    });
    expect(JSON.stringify(eventPublisher.events[0]?.payload)).not.toContain(
      "#passwordInputText",
    );
    expect(JSON.stringify(eventPublisher.events[0]?.payload)).not.toContain(
      "rakuten.example.test",
    );
    expect(operationLogRepository.entries[0]).toMatchObject({
      eventType: OPERATION_LOG_EVENT_TYPES.APPLY_LOTTERY,
      application: null,
      status: "Failure",
      message: "テスト株式会社 のIPO抽選申し込みに失敗しました",
      errorMessage: "selector missing",
    });
    expect(operationLogRepository.entries[0]?.errorMessage).not.toContain(
      "#passwordInputText",
    );
    expect(operationLogRepository.entries[0]?.errorMessage).not.toContain(
      "rakuten.example.test",
    );
  });

  it("uses the result-determination timestamp for failed logs and events", async () => {
    const applicationRepository = new InMemoryApplicationRepository();
    const operationLogRepository = new InMemoryOperationLogRepository();
    const eventPublisher = new InMemoryNotificationPublisher();
    const nowProvider = new SequenceNowProvider([
      "2026-04-10T00:00:00.000Z",
      "2026-04-10T00:00:09.000Z",
    ]);
    const useCase = new ApplyForLotteryUseCase(
      new InMemoryAccountRepository([account]),
      new InMemoryStockRepository([stock]),
      new InMemoryExclusionRepository([]),
      applicationRepository,
      operationLogRepository,
      eventPublisher,
      new ApplicationFailureBrokerPort(),
      nowProvider,
    );

    await useCase.execute({ targetDate: "2026-04-10" });

    expect(operationLogRepository.entries[0]).toMatchObject({
      executedAt: "2026-04-10T00:00:09.000Z",
    });
    expect(eventPublisher.events[0]).toMatchObject({
      payload: {
        failed_at: "2026-04-10T00:00:09.000Z",
      },
    });
  });

  it("does not expose secret-related raw errors in external failure notifications", async () => {
    const applicationRepository = new InMemoryApplicationRepository();
    const operationLogRepository = new InMemoryOperationLogRepository();
    const eventPublisher = new InMemoryNotificationPublisher();
    const useCase = new ApplyForLotteryUseCase(
      new InMemoryAccountRepository([account]),
      new InMemoryStockRepository([stock]),
      new InMemoryExclusionRepository([]),
      applicationRepository,
      operationLogRepository,
      eventPublisher,
      new SecretFailureBrokerPort(),
      { now: () => new Date("2026-04-10T00:00:00.000Z") },
    );

    const output = await useCase.execute({ targetDate: "2026-04-10" });

    expect(output.failedCount).toBe(1);
    expect(eventPublisher.events[0]).toMatchObject({
      eventType: "ApplicationFailed",
      payload: {
        error_message: "IPO抽選の申し込みに失敗しました",
      },
    });
    expect(JSON.stringify(eventPublisher.events[0]?.payload)).not.toContain(
      "super-secret-value",
    );
    expect(JSON.stringify(eventPublisher.events[0]?.payload)).not.toContain(
      "loginPassword",
    );
    expect(operationLogRepository.entries[0]).toMatchObject({
      application: null,
      errorMessage: "secret access failed",
    });
  });

  it("stores sanitized error messages for unexpected workflow failures", async () => {
    const applicationRepository = new InMemoryApplicationRepository();
    const operationLogRepository = new InMemoryOperationLogRepository();
    const eventPublisher = new InMemoryNotificationPublisher();
    const useCase = new ApplyForLotteryUseCase(
      new InMemoryAccountRepository([account]),
      new InMemoryStockRepository([stock]),
      new InMemoryExclusionRepository([]),
      applicationRepository,
      operationLogRepository,
      eventPublisher,
      new UnexpectedFailureBrokerPort(),
      { now: () => new Date("2026-04-10T00:00:00.000Z") },
    );

    const output = await useCase.execute({ targetDate: "2026-04-10" });

    expect(output.failedCount).toBe(1);
    expect(output.results[0]?.reason).toBe(
      "selector missing: #passwordInputText on https://rakuten.example.test/apply?token=secret",
    );
    expect(operationLogRepository.entries[0]).toMatchObject({
      application: null,
      status: "Failure",
      message: "テスト株式会社 のIPO抽選処理中に予期しないエラーが発生しました",
      errorMessage: "selector missing",
    });
    expect(operationLogRepository.entries[0]?.errorMessage).not.toContain(
      "#passwordInputText",
    );
    expect(operationLogRepository.entries[0]?.errorMessage).not.toContain(
      "token=secret",
    );
  });

  it("continues the batch after one account-stock pair throws and returns per-account and per-stock summaries", async () => {
    const applicationRepository = new InMemoryApplicationRepository([
      "stock-1:account-2",
    ]);
    const operationLogRepository = new InMemoryOperationLogRepository();
    const eventPublisher = new InMemoryNotificationPublisher();
    const useCase = new ApplyForLotteryUseCase(
      new InMemoryAccountRepository([account, secondAccount]),
      new InMemoryStockRepository([stock, secondStock]),
      new InMemoryExclusionRepository([]),
      applicationRepository,
      operationLogRepository,
      eventPublisher,
      new SelectiveBrokerPort(),
      { now: () => new Date("2026-04-10T00:00:00.000Z") },
    );

    const output = await useCase.execute({ targetDate: "2026-04-10" });

    expect(output).toMatchObject({
      appliedCount: 1,
      skippedCount: 1,
      failedCount: 2,
    });
    expect(output.results).toEqual([
      {
        accountId: "account-1",
        stockId: "stock-1",
        result: "success",
        reason: null,
        failureCategory: null,
      },
      {
        accountId: "account-1",
        stockId: "stock-2",
        result: "failure",
        reason: "selector missing",
        failureCategory: "application",
      },
      {
        accountId: "account-2",
        stockId: "stock-1",
        result: "already_applied",
        reason: "already applied",
        failureCategory: null,
      },
      {
        accountId: "account-2",
        stockId: "stock-2",
        result: "insufficient_balance",
        reason: "insufficient balance",
        failureCategory: null,
      },
    ]);
    expect(output.accountSummaries).toEqual([
      {
        accountId: "account-1",
        appliedCount: 1,
        skippedCount: 0,
        failedCount: 1,
      },
      {
        accountId: "account-2",
        appliedCount: 0,
        skippedCount: 1,
        failedCount: 1,
      },
    ]);
    expect(output.stockSummaries).toEqual([
      {
        stockId: "stock-1",
        appliedCount: 1,
        skippedCount: 1,
        failedCount: 0,
      },
      {
        stockId: "stock-2",
        appliedCount: 0,
        skippedCount: 0,
        failedCount: 2,
      },
    ]);
    expect(eventPublisher.events.map((event) => event.eventType)).toEqual([
      "ApplicationCompleted",
      "ApplicationFailed",
    ]);
  });
});
