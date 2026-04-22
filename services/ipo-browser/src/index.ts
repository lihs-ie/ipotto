import { logger } from "./logger.js";
import { createApp } from "./server.js";

const port = parseInt(process.env["PORT"] ?? "8081", 10);

const { app, browserManager } = createApp();
const server = app.listen(port, "0.0.0.0", () => {
  logger.info({ event: "server.listening", port });
});

const shutdown = async (signal: string): Promise<void> => {
  logger.info({ event: "server.shutdown.start", signal });
  try {
    await browserManager.closeAll();
  } catch (error) {
    logger.error({ event: "server.shutdown.browser_failed", error });
  }
  server.close(() => {
    process.exit(0);
  });
  // Force exit after 10s if connections linger.
  setTimeout(() => {
    process.exit(1);
  }, 10_000).unref();
};

process.on("SIGTERM", () => {
  void shutdown("SIGTERM");
});
process.on("SIGINT", () => {
  void shutdown("SIGINT");
});
