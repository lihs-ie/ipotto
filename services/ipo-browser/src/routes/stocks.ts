import { Router } from "express";

// Phase 3 Sprint 5.1 — GET /internal/stocks stub. The ipo-info-fetcher and
// ipo-api services probe this endpoint via their BrowserServiceClient to
// collect broker-visible IPO stocks. Phase 4 Sprint 8 replaces the stub
// with a real scraping-driven response; until then we return an empty
// array so callers exercise the full HTTP path without spurious data.
export function stocksRouter(): Router {
  const router = Router();

  router.get("/internal/stocks", (_request, response) => {
    response.json([]);
  });

  return router;
}
