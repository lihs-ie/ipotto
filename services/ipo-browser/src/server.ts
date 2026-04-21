import express, { type Express } from "express";

import { accountsRouter } from "./routes/accounts.js";
import { healthRouter } from "./routes/health.js";
import { lotteryResultsRouter } from "./routes/lottery-results.js";
import { stocksRouter } from "./routes/stocks.js";

// Phase 3 Sprint 5.1 — single place that composes every HTTP route the
// ipo-browser service exposes. Entry point (index.ts) is a thin wrapper
// that calls `createApp()` and `listen()`; unit / e2e tests can build the
// same app without starting a server.
export function createApp(): Express {
  const app = express();
  app.use(express.json());
  app.use(healthRouter());
  app.use(accountsRouter());
  app.use(stocksRouter());
  app.use(lotteryResultsRouter());
  return app;
}
