import { describe, expect, it, vi } from "vitest";

import { FirestoreIpoStockRepository } from "./firestore-ipo-stock-repository.js";

describe("FirestoreIpoStockRepository", () => {
  it("returns stocks within the target period and falls back to the minimum price", async () => {
    const collection = vi.fn().mockReturnValue({
      get: vi.fn().mockResolvedValue({
        docs: [
          {
            data: () => ({
              identifier: "stock-1",
              companyProfile: { companyName: "対象IPO" },
              schedule: {
                bookBuildingPeriod: {
                  startDate: "2026-04-10T00:00:00.000Z",
                  endDate: "2026-04-12T00:00:00.000Z",
                },
              },
              pricing: {
                offerPrice: null,
                priceRange: { minimumPrice: 600 },
              },
              offering: {
                numberOfOfferedShares: 100,
              },
            }),
          },
          {
            data: () => ({
              identifier: "stock-2",
              companyProfile: { companyName: "期間外IPO" },
              schedule: {
                bookBuildingPeriod: {
                  startDate: "2026-04-01T00:00:00.000Z",
                  endDate: "2026-04-02T00:00:00.000Z",
                },
              },
              pricing: {
                offerPrice: 700,
                priceRange: { minimumPrice: 650 },
              },
              offering: {
                numberOfOfferedShares: 200,
              },
            }),
          },
        ],
      }),
    });
    const repository = new FirestoreIpoStockRepository({
      collection,
    } as never);

    await expect(
      repository.findInBookBuildingPeriod("2026-04-11"),
    ).resolves.toEqual([
      {
        identifier: "stock-1",
        companyName: "対象IPO",
        price: 600,
        shares: 100,
        bookBuildingStartDate: "2026-04-10",
        bookBuildingEndDate: "2026-04-12",
      },
    ]);
  });
});
