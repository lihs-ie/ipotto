import { describe, expect, it, vi } from "vitest";

import { FirestoreExclusionRepository } from "./firestore-exclusion-repository.js";

describe("FirestoreExclusionRepository", () => {
  it("maps exclusion documents from Firestore", async () => {
    const collection = vi.fn().mockReturnValue({
      get: vi.fn().mockResolvedValue({
        docs: [
          {
            data: () => ({
              identifier: "exclusion-1",
              companyName: "除外IPO",
              reason: "manual",
            }),
          },
        ],
      }),
    });
    const repository = new FirestoreExclusionRepository({
      collection,
    } as never);

    await expect(repository.findAll()).resolves.toEqual([
      {
        identifier: "exclusion-1",
        companyName: "除外IPO",
        reason: "manual",
      },
    ]);
  });
});
