import { describe, expect, it, vi } from "vitest";

import type { DependencyContainer } from "../../infrastructure/dependency-container.js";
import { createApplyMessageHandler } from "./pubsub-message-handler.js";

describe("createApplyMessageHandler", () => {
  it("accepts direct JSON input", async () => {
    const execute = vi.fn().mockResolvedValue({
      appliedCount: 1,
      skippedCount: 0,
      failedCount: 0,
      results: [],
    });
    const handler = createApplyMessageHandler({
      applyForLotteryUseCase: () => ({ execute }),
    } as unknown as DependencyContainer);

    const status = vi.fn().mockReturnThis();
    const json = vi.fn();

    await handler(
      { body: { targetDate: "2026-04-10" } } as never,
      { status, json } as never,
      vi.fn(),
    );

    expect(execute).toHaveBeenCalledWith({ targetDate: "2026-04-10" });
    expect(status).toHaveBeenCalledWith(200);
  });

  it("accepts Pub/Sub push input", async () => {
    const execute = vi.fn().mockResolvedValue({
      appliedCount: 0,
      skippedCount: 0,
      failedCount: 0,
      results: [],
    });
    const handler = createApplyMessageHandler({
      applyForLotteryUseCase: () => ({ execute }),
    } as unknown as DependencyContainer);

    const status = vi.fn().mockReturnThis();
    const json = vi.fn();

    await handler(
      {
        body: {
          message: {
            data: Buffer.from(
              JSON.stringify({ targetDate: "2026-04-10" }),
              "utf8",
            ).toString("base64"),
          },
        },
      } as never,
      { status, json } as never,
      vi.fn(),
    );

    expect(execute).toHaveBeenCalledWith({ targetDate: "2026-04-10" });
    expect(status).toHaveBeenCalledWith(200);
  });
});
