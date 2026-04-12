import { mkdtemp, utimes } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { BrowserSessionStorage } from "./browser-session-storage.js";

describe("BrowserSessionStorage", () => {
  it("cleans up session directories older than the retention period", async () => {
    const baseDirectory = await mkdtemp(
      path.join(tmpdir(), "ipo-browser-session-storage-"),
    );
    const storage = new BrowserSessionStorage(baseDirectory);
    const staleDirectory = await storage.ensureUserDataDirectory("stale-account");
    const freshDirectory = await storage.ensureUserDataDirectory("fresh-account");

    await utimes(
      staleDirectory,
      new Date("2026-04-09T23:00:00.000Z"),
      new Date("2026-04-09T23:00:00.000Z"),
    );
    await utimes(
      freshDirectory,
      new Date("2026-04-10T23:30:00.000Z"),
      new Date("2026-04-10T23:30:00.000Z"),
    );

    await expect(
      storage.cleanupExpiredSessions(24, {
        now: () => new Date("2026-04-11T00:00:00.000Z"),
      }),
    ).resolves.toBe(1);

    await expect(storage.ensureUserDataDirectory("fresh-account")).resolves.toBe(
      freshDirectory,
    );
  });
});
