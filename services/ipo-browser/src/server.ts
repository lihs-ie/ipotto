import express, { type Express } from "express";

import { BrowserManager } from "./browser/manager.js";
import { accountsRouter } from "./routes/accounts.js";
import { healthRouter } from "./routes/health.js";
import { lotteryResultsRouter } from "./routes/lottery-results.js";
import { stocksRouter } from "./routes/stocks.js";

// Phase 3 Sprint 5.1 — HTTP server factory. Sprint 5.2 adds the
// BrowserManager so the Playwright contexts are owned at application
// scope and can be torn down gracefully on shutdown.

export type CreateAppOptions = {
  browserManager?: BrowserManager;
};

export type CreatedApp = {
  app: Express;
  browserManager: BrowserManager;
};

export function createApp(options: CreateAppOptions = {}): CreatedApp {
  const browserManager = options.browserManager ?? new BrowserManager();
  const app = express();
  app.use(express.json());
  app.use(healthRouter());
  app.use(accountsRouter(browserManager));
  app.use(stocksRouter());
  app.use(lotteryResultsRouter());
  return { app, browserManager };
}
