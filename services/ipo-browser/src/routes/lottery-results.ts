import { Router, type Request, type Response } from "express";

// Phase 3 Sprint 5.1 — POST /internal/lottery-results/check stub.
// ipo-result-checker and ipo-api delegate here to confirm a specific
// stock's lottery outcome. Phase 3 Sprint 7 swaps this for the real
// Playwright-driven result-page scrape; the stub response shape matches
// `BrowserLotteryResultResponse` in both Rust clients.
type LotteryResultCheckRequest = {
  credential?: Record<string, unknown>;
  stockIdentifier?: string;
};

export function lotteryResultsRouter(): Router {
  const router = Router();

  router.post(
    "/internal/lottery-results/check",
    (request: Request, response: Response) => {
      const body = request.body as LotteryResultCheckRequest | undefined;
      if (body === undefined || body.stockIdentifier === undefined) {
        response.status(400).json({
          error: "stockIdentifier is required",
        });
        return;
      }
      response.json({ result: null });
    },
  );

  return router;
}
