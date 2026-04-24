import pino from "pino";

/// Structured JSON logger for ipo-browser. Writes to stdout with
/// `severity` instead of `level` to match Cloud Logging's expected
/// payload shape, and redacts known credential paths before the entry
/// is serialised. Environment override: `LOG_LEVEL` (default `info`).
export const logger = pino({
  level: process.env.LOG_LEVEL ?? "info",
  messageKey: "message",
  base: { service: "ipo-browser" },
  formatters: {
    level: (label) => ({ severity: label.toUpperCase() }),
  },
  redact: {
    paths: [
      "loginPassword",
      "tradingPassword",
      "mailPassword",
      "password",
      "authorization",
      "headers.authorization",
      "*.loginPassword",
      "*.tradingPassword",
      "*.mailPassword",
      "*.password",
    ],
    censor: "[Redacted]",
  },
});
