import { readFile } from "node:fs/promises";
import { createServer, type Server } from "node:http";
import { tmpdir } from "node:os";
import { extname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import type { Express } from "express";

import type {
  ActiveSecuritiesAccountRepository,
  ExclusionRepository,
  IpoStockRepository,
  LotteryApplicationRepository,
  NotificationEventPublisher,
  OperationLogRepository,
} from "../application/use-cases/apply-for-lottery-use-case.js";
import type { ActiveSecuritiesAccount } from "../domain/account-credential.js";
import type {
  LotteryApplicationRecord,
  NotificationEventEnvelope,
  OperationLogEntry,
} from "../domain/notification-event.js";
import type { ExclusionEntry, TargetIpoStock } from "../domain/ipo-stock.js";

const FIXTURE_ROOT = fileURLToPath(
  new URL("../../tests/fixtures/html/", import.meta.url),
);

/**
 * Running local HTTP server used by smoke and feature tests.
 */
export interface RunningHttpServer {
  /**
   * Base URL for the server.
   */
  readonly baseUrl: string;

  /**
   * Closes the server.
   */
  close(): Promise<void>;
}

/**
 * In-memory account repository used by system-style tests.
 */
export class InMemoryActiveSecuritiesAccountRepository
  implements ActiveSecuritiesAccountRepository
{
  /**
   * Creates the repository.
   */
  public constructor(
    private readonly accounts: readonly ActiveSecuritiesAccount[],
  ) {}

  /**
   * Returns the configured active accounts.
   */
  public async findActive(): Promise<readonly ActiveSecuritiesAccount[]> {
    return this.accounts;
  }
}

/**
 * In-memory stock repository used by system-style tests.
 */
export class InMemoryIpoStockRepository implements IpoStockRepository {
  /**
   * Creates the repository.
   */
  public constructor(
    private readonly stocks: readonly TargetIpoStock[],
  ) {}

  /**
   * Returns stocks in the given book-building period.
   */
  public async findInBookBuildingPeriod(
    targetDate: string,
  ): Promise<readonly TargetIpoStock[]> {
    return this.stocks.filter(
      (stock) =>
        stock.bookBuildingStartDate <= targetDate &&
        targetDate <= stock.bookBuildingEndDate,
    );
  }
}

/**
 * In-memory exclusion repository used by system-style tests.
 */
export class InMemoryExclusionRepository implements ExclusionRepository {
  /**
   * Creates the repository.
   */
  public constructor(
    private readonly exclusions: readonly ExclusionEntry[] = [],
  ) {}

  /**
   * Returns all exclusions.
   */
  public async findAll(): Promise<readonly ExclusionEntry[]> {
    return this.exclusions;
  }
}

/**
 * In-memory lottery application repository used by system-style tests.
 */
export class InMemoryLotteryApplicationRepository
  implements LotteryApplicationRepository
{
  private readonly records: LotteryApplicationRecord[] = [];

  /**
   * Returns whether an application already exists for the stock/account pair.
   */
  public async existsByStockAndAccount(
    stockId: string,
    accountId: string,
  ): Promise<boolean> {
    return this.records.some(
      (record) =>
        record.stock === stockId && record.securitiesAccount === accountId,
    );
  }

  /**
   * Saves a lottery application record.
   */
  public async save(record: LotteryApplicationRecord): Promise<void> {
    this.records.push(record);
  }

  /**
   * Returns saved records for assertions.
   */
  public savedRecords(): readonly LotteryApplicationRecord[] {
    return this.records;
  }
}

/**
 * In-memory operation log repository used by system-style tests.
 */
export class InMemoryOperationLogRepository implements OperationLogRepository {
  private readonly entries: OperationLogEntry[] = [];

  /**
   * Saves an operation log entry.
   */
  public async save(entry: OperationLogEntry): Promise<void> {
    this.entries.push(entry);
  }

  /**
   * Returns saved entries for assertions.
   */
  public savedEntries(): readonly OperationLogEntry[] {
    return this.entries;
  }
}

/**
 * In-memory notification event publisher used by system-style tests.
 */
export class InMemoryNotificationEventPublisher
  implements NotificationEventPublisher
{
  private readonly events: NotificationEventEnvelope[] = [];

  /**
   * Publishes an event into the in-memory buffer.
   */
  public async publish(event: NotificationEventEnvelope): Promise<void> {
    this.events.push(event);
  }

  /**
   * Returns published events for assertions.
   */
  public publishedEvents(): readonly NotificationEventEnvelope[] {
    return this.events;
  }
}

/**
 * Starts a static HTTP server backed by the browser test fixtures.
 */
export async function startFixtureHttpServer(): Promise<RunningHttpServer> {
  const server = createServer(async (request, response) => {
    const requestUrl = new URL(
      request.url ?? "/",
      "http://127.0.0.1",
    );
    const pathname =
      requestUrl.pathname === "/" ? "/index.html" : requestUrl.pathname;
    const filePath = resolve(FIXTURE_ROOT, `.${pathname}`);

    if (!filePath.startsWith(FIXTURE_ROOT)) {
      response.statusCode = 403;
      response.end("forbidden");
      return;
    }

    try {
      const body = await readFile(filePath);
      response.statusCode = 200;
      response.setHeader("content-type", contentTypeFor(filePath));
      response.end(body);
    } catch {
      response.statusCode = 404;
      response.end("not found");
    }
  });

  const address = await listen(server);
  return {
    baseUrl: `http://127.0.0.1:${address.port}`,
    close: async () => closeServer(server),
  };
}

/**
 * Starts an Express application on an ephemeral local port.
 */
export async function startExpressHttpServer(
  app: Express,
): Promise<RunningHttpServer> {
  const server = app.listen(0, "127.0.0.1");
  const address = await new Promise<{ readonly port: number }>((resolveAddress, reject) => {
    server.once("listening", () => {
      const serverAddress = server.address();
      if (
        serverAddress !== null &&
        typeof serverAddress === "object" &&
        "port" in serverAddress
      ) {
        resolveAddress({ port: serverAddress.port });
        return;
      }
      reject(new Error("express server address is unavailable"));
    });
    server.once("error", reject);
  });

  return {
    baseUrl: `http://127.0.0.1:${address.port}`,
    close: async () =>
      new Promise<void>((resolveClose, reject) => {
        server.close((error) => {
          if (error !== undefined && error !== null) {
            reject(error);
            return;
          }
          resolveClose();
        });
      }),
  };
}

/**
 * Returns a unique temporary directory path for browser-session tests.
 */
export function createTemporaryBrowserSessionBaseDir(): string {
  return resolve(
    tmpdir(),
    "ipotto-ipo-browser-tests",
    crypto.randomUUID(),
  );
}

/**
 * Returns the fixture root directory used by HTTP-backed tests.
 */
export function fixtureRootDirectory(): string {
  return FIXTURE_ROOT;
}

/**
 * Resolves a content type from a file extension.
 */
function contentTypeFor(filePath: string): string {
  switch (extname(filePath)) {
    case ".html":
      return "text/html; charset=utf-8";
    case ".json":
      return "application/json; charset=utf-8";
    default:
      return "application/octet-stream";
  }
}

/**
 * Starts a generic Node HTTP server.
 */
function listen(server: Server): Promise<{ readonly port: number }> {
  return new Promise((resolveAddress, reject) => {
    server.once("listening", () => {
      const address = server.address();
      if (address !== null && typeof address === "object" && "port" in address) {
        resolveAddress({ port: address.port });
        return;
      }
      reject(new Error("fixture server address is unavailable"));
    });
    server.once("error", reject);
    server.listen(0, "127.0.0.1");
  });
}

/**
 * Closes a Node HTTP server.
 */
function closeServer(server: Server): Promise<void> {
  return new Promise((resolveClose, reject) => {
    server.close((error) => {
      if (error !== undefined && error !== null) {
        reject(error);
        return;
      }
      resolveClose();
    });
  });
}
