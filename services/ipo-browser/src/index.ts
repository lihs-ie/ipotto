import { createApp } from "./server.js";

const port = parseInt(process.env["PORT"] ?? "8081", 10);

const { app, browserManager } = createApp();
const server = app.listen(port, "0.0.0.0", () => {
  console.log(`ipo-browser listening on port ${port}`);
});

const shutdown = async (signal: string): Promise<void> => {
  console.log(`received ${signal}, closing browser contexts and HTTP server`);
  try {
    await browserManager.closeAll();
  } catch (error) {
    console.error("browser shutdown failed", error);
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
