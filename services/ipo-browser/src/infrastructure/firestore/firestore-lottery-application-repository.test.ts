import { describe, expect, it, vi } from "vitest";

import { FirestoreLotteryApplicationRepository } from "./firestore-lottery-application-repository.js";

describe("FirestoreLotteryApplicationRepository", () => {
  it("checks whether an application already exists", async () => {
    const get = vi.fn().mockResolvedValue({ empty: false });
    const limit = vi.fn().mockReturnValue({ get });
    const secondWhere = vi.fn().mockReturnValue({ limit });
    const firstWhere = vi.fn().mockReturnValue({ where: secondWhere });
    const collection = vi.fn().mockReturnValue({ where: firstWhere });
    const repository = new FirestoreLotteryApplicationRepository({
      collection,
    } as never);

    await expect(
      repository.existsByStockAndAccount("stock-1", "account-1"),
    ).resolves.toBe(true);
    expect(firstWhere).toHaveBeenCalledWith("stock", "==", "stock-1");
    expect(secondWhere).toHaveBeenCalledWith(
      "securitiesAccount",
      "==",
      "account-1",
    );
  });

  it("saves an application record by identifier", async () => {
    const set = vi.fn().mockResolvedValue(undefined);
    const doc = vi.fn().mockReturnValue({ set });
    const collection = vi.fn().mockReturnValue({ doc });
    const repository = new FirestoreLotteryApplicationRepository({
      collection,
    } as never);
    const record = {
      identifier: "application-1",
      stock: "stock-1",
      securitiesAccount: "account-1",
      appliedOrder: {
        shares: 100,
        price: 600,
        orderedAt: "2026-04-13T00:00:00.000Z",
      },
      lotteryOutcome: null,
      status: "Applied",
      createdAt: "2026-04-13T00:00:00.000Z",
      updatedAt: "2026-04-13T00:00:00.000Z",
    } as const;

    await repository.save(record);

    expect(collection).toHaveBeenCalledWith("lottery_applications");
    expect(doc).toHaveBeenCalledWith("application-1");
    expect(set).toHaveBeenCalledWith(record);
  });
});
