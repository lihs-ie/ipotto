import express, { type Express } from "express";

import { BrowserManager } from "./browser/manager.js";
import {
  createNotificationPublisher,
  type NotificationPublisher,
} from "./notifications/publisher.js";
import { accountsRouter } from "./routes/accounts.js";
import { healthRouter } from "./routes/health.js";
import { lotteryResultsRouter } from "./routes/lottery-results.js";
import { stocksRouter } from "./routes/stocks.js";

// Phase 3 Sprint 5.1 — HTTP server factory. Sprint 5.2 adds the
// BrowserManager so the Playwright contexts are owned at application
// scope. Sprint 6.5 wires in the NotificationPublisher so the
// account-test handler can publish OperationErrorOccurred events when
// the 2FA flow fails.

export type CreateAppOptions = {
  browserManager?: BrowserManager;
  notificationPublisher?: NotificationPublisher;
};

export type CreatedApp = {
  app: Express;
  browserManager: BrowserManager;
  notificationPublisher: NotificationPublisher;
};

export function createApp(options: CreateAppOptions = {}): CreatedApp {
  const browserManager = options.browserManager ?? new BrowserManager();
  const notificationPublisher =
    options.notificationPublisher ?? createNotificationPublisher();
  const app = express();
  app.use(express.json());
  app.use(healthRouter());
  app.use(accountsRouter(browserManager, notificationPublisher));
  app.use(stocksRouter());
  app.use(lotteryResultsRouter());
  return { app, browserManager, notificationPublisher };
}
