import { describe, expect, it, vi } from "vitest";

import {
  ApplyForLotteryValidationError,
} from "../../application/use-cases/apply-for-lottery-use-case.js";
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

  it("returns 200 for partial failures because the workflow completed", async () => {
    const execute = vi.fn().mockResolvedValue({
      appliedCount: 1,
      skippedCount: 1,
      failedCount: 1,
      results: [
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
      ],
      accountSummaries: [],
      stockSummaries: [],
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

    expect(status).toHaveBeenCalledWith(200);
    expect(json).toHaveBeenCalledWith({
      appliedCount: 1,
      skippedCount: 1,
      failedCount: 1,
      results: [
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
      ],
      accountSummaries: [],
      stockSummaries: [],
    });
  });

  it("returns 200 even when all attempts failed because the workflow still completed", async () => {
    const execute = vi.fn().mockResolvedValue({
      appliedCount: 0,
      skippedCount: 0,
      failedCount: 2,
      results: [
        {
          accountId: "account-1",
          stockId: "stock-1",
          result: "failure",
          reason: "selector missing",
          failureCategory: "application",
        },
      ],
      accountSummaries: [],
      stockSummaries: [],
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

    expect(status).toHaveBeenCalledWith(200);
  });

  it("returns 400 for invalid input", async () => {
    const execute = vi.fn();
    const handler = createApplyMessageHandler({
      applyForLotteryUseCase: () => ({ execute }),
    } as unknown as DependencyContainer);

    const status = vi.fn().mockReturnThis();
    const json = vi.fn();

    await handler(
      { body: { targetDate: "2026/04/10" } } as never,
      { status, json } as never,
      vi.fn(),
    );

    expect(execute).not.toHaveBeenCalled();
    expect(status).toHaveBeenCalledWith(400);
    expect(json).toHaveBeenCalledWith({
      code: "BAD_REQUEST",
      message: "targetDate must be in YYYY-MM-DD format",
    });
  });

  it("returns 400 when accountIds or stockIds are supplied before support exists", async () => {
    const execute = vi.fn();
    const handler = createApplyMessageHandler({
      applyForLotteryUseCase: () => ({ execute }),
    } as unknown as DependencyContainer);

    const status = vi.fn().mockReturnThis();
    const json = vi.fn();

    await handler(
      { body: { targetDate: "2026-04-10", accountIds: ["account-1"] } } as never,
      { status, json } as never,
      vi.fn(),
    );

    expect(execute).not.toHaveBeenCalled();
    expect(status).toHaveBeenCalledWith(400);
    expect(json).toHaveBeenCalledWith({
      code: "BAD_REQUEST",
      message: "accountIds is not supported yet",
    });
  });

  it("returns 400 when the use case raises a validation error", async () => {
    const execute = vi
      .fn()
      .mockRejectedValue(
        new ApplyForLotteryValidationError("targetDate must not be in the future"),
      );
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

    expect(status).toHaveBeenCalledWith(400);
    expect(json).toHaveBeenCalledWith({
      code: "BAD_REQUEST",
      message: "targetDate must not be in the future",
    });
  });

  it("returns 503 when the workflow crashes before completion", async () => {
    const execute = vi.fn().mockRejectedValue(new Error("firestore unavailable"));
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

    expect(status).toHaveBeenCalledWith(503);
    expect(json).toHaveBeenCalledWith({
      code: "SERVICE_UNAVAILABLE",
      message: "apply workflow failed before completion",
    });
  });
});
