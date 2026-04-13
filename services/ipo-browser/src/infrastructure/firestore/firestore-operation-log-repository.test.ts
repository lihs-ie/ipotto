import { describe, expect, it, vi } from "vitest";

import { FirestoreOperationLogRepository } from "./firestore-operation-log-repository.js";

describe("FirestoreOperationLogRepository", () => {
  it("saves the operation log entry by identifier", async () => {
    const set = vi.fn().mockResolvedValue(undefined);
    const doc = vi.fn().mockReturnValue({ set });
    const collection = vi.fn().mockReturnValue({ doc });
    const repository = new FirestoreOperationLogRepository({
      collection,
    } as never);
    const entry = {
      identifier: "log-1",
      application: null,
      eventType: "ApplyLottery",
      serviceName: "ipo-browser",
      status: "Failure",
      message: "IPO 抽選申込に失敗しました",
      errorMessage: "application failed",
      executedAt: "2026-04-13T00:00:00.000Z",
    } as const;

    await repository.save(entry);

    expect(collection).toHaveBeenCalledWith("operation_logs");
    expect(doc).toHaveBeenCalledWith("log-1");
    expect(set).toHaveBeenCalledWith(entry);
  });
});
