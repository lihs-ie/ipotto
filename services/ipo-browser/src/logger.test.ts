import pino from "pino";
import { describe, expect, it } from "vitest";

const captureLogs = (): {
  lines: string[];
  logger: pino.Logger;
} => {
  const lines: string[] = [];
  const stream = {
    write(value: string) {
      lines.push(value);
    },
  };
  const logger = pino(
    {
      level: "info",
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
          "*.loginPassword",
          "*.tradingPassword",
          "*.mailPassword",
          "*.password",
        ],
        censor: "[Redacted]",
      },
    },
    stream,
  );
  return { lines, logger };
};

describe("logger", () => {
  it("redacts credentials recursively", () => {
    const { lines, logger } = captureLogs();
    logger.info({
      event: "test",
      loginPassword: "secret",
      nested: { mailPassword: "mail-secret" },
    });
    const firstLine = lines[0];
    if (firstLine === undefined) throw new Error("expected a log entry");
    const parsed = JSON.parse(firstLine);
    expect(parsed.loginPassword).toBe("[Redacted]");
    expect(parsed.nested.mailPassword).toBe("[Redacted]");
    expect(parsed.severity).toBe("INFO");
    expect(parsed.service).toBe("ipo-browser");
  });

  it("preserves non-sensitive fields", () => {
    const { lines, logger } = captureLogs();
    logger.info({ event: "health", port: 8081 });
    const firstLine = lines[0];
    if (firstLine === undefined) throw new Error("expected a log entry");
    const parsed = JSON.parse(firstLine);
    expect(parsed.event).toBe("health");
    expect(parsed.port).toBe(8081);
  });

  it("emits severity in upper case for Cloud Logging", () => {
    const { lines, logger } = captureLogs();
    logger.error({ event: "failure" });
    const firstLine = lines[0];
    if (firstLine === undefined) throw new Error("expected a log entry");
    const parsed = JSON.parse(firstLine);
    expect(parsed.severity).toBe("ERROR");
  });
});
