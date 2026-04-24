import { promises as fs } from "node:fs";
import { join } from "node:path";

import { chromium, type BrowserContext } from "playwright";

// Phase 3 Sprint 5.2 — Playwright browser manager with per-session
// persistent contexts. A session is keyed by an opaque string (today:
// the broker loginId) so the same `user data directory` is reused
// across requests, avoiding the cost of logging in again on every API
// call and preserving cookies / storage like Rakuten's "device" token.
//
// Contexts are cached in-process; callers should reach for `acquire`
// on every request instead of keeping their own handle, and `closeAll`
// is wired into the HTTP server's graceful shutdown.

const DEFAULT_SESSION_DIR = "/tmp/ipo-browser-sessions";

export type BrowserManagerOptions = {
  sessionBaseDir?: string;
  headless?: boolean;
};

export class BrowserManager {
  private readonly contexts = new Map<string, Promise<BrowserContext>>();
  private readonly sessionBaseDir: string;
  private readonly headless: boolean;

  constructor(options: BrowserManagerOptions = {}) {
    this.sessionBaseDir =
      options.sessionBaseDir ??
      process.env["BROWSER_SESSION_DIR"] ??
      DEFAULT_SESSION_DIR;
    this.headless = options.headless ?? true;
  }

  /**
   * Returns a persistent BrowserContext for the supplied session key,
   * launching a fresh one on first access. Concurrent calls with the
   * same key share the same launch Promise so we don't race on the
   * filesystem lock Playwright takes on the user data directory.
   */
  async acquire(sessionKey: string): Promise<BrowserContext> {
    const existing = this.contexts.get(sessionKey);
    if (existing !== undefined) {
      return existing;
    }
    const launching = this.launchContext(sessionKey);
    this.contexts.set(sessionKey, launching);
    try {
      return await launching;
    } catch (error) {
      // Failed launch — make sure the next call retries from scratch.
      this.contexts.delete(sessionKey);
      throw error;
    }
  }

  async release(sessionKey: string): Promise<void> {
    const existing = this.contexts.get(sessionKey);
    if (existing === undefined) {
      return;
    }
    this.contexts.delete(sessionKey);
    const context = await existing;
    await context.close();
  }

  async closeAll(): Promise<void> {
    const pending = Array.from(this.contexts.values());
    this.contexts.clear();
    for (const contextPromise of pending) {
      try {
        const context = await contextPromise;
        await context.close();
      } catch {
        // best-effort on shutdown
      }
    }
  }

  private async launchContext(sessionKey: string): Promise<BrowserContext> {
    const userDataDir = join(
      this.sessionBaseDir,
      sanitiseSessionKey(sessionKey),
    );
    await fs.mkdir(userDataDir, { recursive: true });
    return chromium.launchPersistentContext(userDataDir, {
      headless: this.headless,
      args: ["--no-sandbox", "--disable-dev-shm-usage"],
    });
  }
}

function sanitiseSessionKey(value: string): string {
  // Strip any path-affecting characters so the session key can be any
  // opaque string from the caller (loginId / UID / etc.).
  return value.replace(/[^a-zA-Z0-9._-]/g, "_");
}
