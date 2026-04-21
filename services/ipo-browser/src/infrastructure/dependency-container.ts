import { Firestore } from "@google-cloud/firestore";
import { PubSub } from "@google-cloud/pubsub";
import { SecretManagerServiceClient } from "@google-cloud/secret-manager";

import {
  ApplyForLotteryUseCase,
  type ActiveSecuritiesAccountRepository,
  type BrokerOperationPort,
  type ExclusionRepository,
  type IpoStockRepository,
  type LotteryApplicationRepository,
  type NotificationEventPublisher,
  type OperationLogRepository,
} from "../application/use-cases/apply-for-lottery-use-case.js";
import { readAppConfig } from "./config/app-config.js";
import { FirestoreActiveSecuritiesAccountRepository } from "./firestore/firestore-active-securities-account-repository.js";
import { FirestoreExclusionRepository } from "./firestore/firestore-exclusion-repository.js";
import { FirestoreIpoStockRepository } from "./firestore/firestore-ipo-stock-repository.js";
import { FirestoreLotteryApplicationRepository } from "./firestore/firestore-lottery-application-repository.js";
import { FirestoreOperationLogRepository } from "./firestore/firestore-operation-log-repository.js";
import { ImapRakutenAuthMailSource } from "./mail/imap-rakuten-auth-mail-source.js";
import {
  PollingImageAuthenticationKeywordProvider,
} from "./mail/polling-image-authentication-keyword-provider.js";
import { PubSubNotificationEventPublisher } from "./pubsub/pubsub-notification-event-publisher.js";
import {
  FixtureRakutenNavigationTargetResolver,
  ProductionRakutenNavigationTargetResolver,
} from "./rakuten/rakuten-navigation-target-resolver.js";
import {
  RakutenBrokerAdapter,
  StaticImageAuthenticationKeywordProvider,
} from "./rakuten/rakuten-broker-adapter.js";
import { MockStockCatalogClient } from "./rakuten/mock-stock-catalog-client.js";
import { PageFactory } from "./rakuten/page-factory.js";
import { SecretManagerAccountCredentialStore } from "./secrets/secret-manager-account-credential-store.js";
import { BrowserSessionStorage } from "./session/browser-session-storage.js";

/**
 * Runtime dependency container for ipo-browser.
 */
export interface DependencyContainer {
  /**
   * Returns the DD-101 use case.
   */
  applyForLotteryUseCase(): ApplyForLotteryUseCase;

  /**
   * Returns the broker adapter.
   */
  brokerPort(): BrokerOperationPort;

  /**
   * Returns the mock stock catalog client.
   */
  stockCatalogClient(): MockStockCatalogClient;
}

class RuntimeDependencyContainer implements DependencyContainer {
  /**
   * Creates the runtime container.
   */
  public constructor(
    private readonly applyUseCase: ApplyForLotteryUseCase,
    private readonly broker: BrokerOperationPort,
    private readonly stockCatalog: MockStockCatalogClient,
  ) {}

  /**
   * Returns the DD-101 use case.
   */
  public applyForLotteryUseCase(): ApplyForLotteryUseCase {
    return this.applyUseCase;
  }

  /**
   * Returns the broker adapter.
   */
  public brokerPort(): BrokerOperationPort {
    return this.broker;
  }

  /**
   * Returns the stock catalog client.
   */
  public stockCatalogClient(): MockStockCatalogClient {
    return this.stockCatalog;
  }
}

/**
 * Creates the production dependency container.
 */
export function createDependencyContainer(): DependencyContainer {
  const config = readAppConfig();
  const firestore = new Firestore({ projectId: config.gcpProjectId });
  const pubsub = new PubSub({ projectId: config.gcpProjectId });
  const secretManager = new SecretManagerServiceClient();
  const credentialStore = new SecretManagerAccountCredentialStore(
    secretManager,
    config.gcpProjectId,
  );

  const accountRepository = new FirestoreActiveSecuritiesAccountRepository(
    firestore,
    credentialStore,
  );
  const stockRepository = new FirestoreIpoStockRepository(firestore);
  const exclusionRepository = new FirestoreExclusionRepository(firestore);
  const applicationRepository = new FirestoreLotteryApplicationRepository(firestore);
  const operationLogRepository = new FirestoreOperationLogRepository(firestore);
  const eventPublisher = new PubSubNotificationEventPublisher(
    pubsub,
    config.notificationTopic,
  );
  const sessionStorage = new BrowserSessionStorage(config.browserSessionBaseDir);
  void sessionStorage
    .cleanupExpiredSessions(config.browserSessionRetentionHours)
    .catch(() => undefined);
  const keywordProvider =
    config.rakutenImageAuthenticationKeywords !== null
      ? new StaticImageAuthenticationKeywordProvider(
          config.rakutenImageAuthenticationKeywords,
        )
      : new PollingImageAuthenticationKeywordProvider(
          // DD-101 currently supports IMAP-based mail retrieval only.
          new ImapRakutenAuthMailSource(),
          {
            pollingIntervalMs: 3_000,
            timeoutMs: 120_000,
          },
        );
  const navigationTargetResolver =
    config.mockServerUrl === null
      ? new ProductionRakutenNavigationTargetResolver(
          config.rakutenLoginPageUrl,
          config.rakutenIpoListPageUrl,
          config.rakutenApplicationPageUrl,
        )
      : new FixtureRakutenNavigationTargetResolver(config.mockServerUrl);
  const broker = new RakutenBrokerAdapter(
    new PageFactory(),
    sessionStorage,
    keywordProvider,
    navigationTargetResolver,
    {
      mockLotteryResult: config.mockLotteryResult,
    },
  );
  const stockCatalog = new MockStockCatalogClient(config.stockCatalogUrl);

  return fromComponents(
    accountRepository,
    stockRepository,
    exclusionRepository,
    applicationRepository,
    operationLogRepository,
    eventPublisher,
    broker,
    stockCatalog,
  );
}

/**
 * Creates a container from explicit components for tests.
 */
export function fromComponents(
  accountRepository: ActiveSecuritiesAccountRepository,
  stockRepository: IpoStockRepository,
  exclusionRepository: ExclusionRepository,
  applicationRepository: LotteryApplicationRepository,
  operationLogRepository: OperationLogRepository,
  eventPublisher: NotificationEventPublisher,
  broker: BrokerOperationPort,
  stockCatalog: MockStockCatalogClient,
): DependencyContainer {
  const nowProvider = {
    now: () => new Date(),
  };

  return new RuntimeDependencyContainer(
    new ApplyForLotteryUseCase(
      accountRepository,
      stockRepository,
      exclusionRepository,
      applicationRepository,
      operationLogRepository,
      eventPublisher,
      broker,
      nowProvider,
    ),
    broker,
    stockCatalog,
  );
}
