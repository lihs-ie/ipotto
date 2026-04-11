import { describe, expect, it } from "vitest";

import type { ActiveSecuritiesAccount } from "../../domain/account-credential.js";
import type { ExclusionEntry, TargetIpoStock } from "../../domain/ipo-stock.js";
import type {
  LotteryApplicationRecord,
  NotificationEventEnvelope,
  OperationLogEntry,
} from "../../domain/notification-event.js";
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

  public async existsByStockAndAccount(): Promise<boolean> {
    return false;
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

describe("ApplyForLotteryUseCase", () => {
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
    expect(applicationRepository.records).toHaveLength(1);
    expect(eventPublisher.events[0]?.eventType).toBe("ApplicationCompleted");
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
    expect(applicationRepository.records).toHaveLength(0);
  });
});
