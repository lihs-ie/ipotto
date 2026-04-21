import { describe, expect, it } from "vitest";

import { pollForImageAuthenticationKeyword } from "./polling.js";
import {
  MailRetrievalTimeoutError,
  type MailPollingConfig,
} from "./types.js";

const config: MailPollingConfig = Object.freeze({
  pollingIntervalMs: 100,
  maxTimeoutMs: 500,
});

type Tick = { kind: "supplier" } | { kind: "sleep"; ms: number };

function createFakeClock(initialNow: number) {
  const state = { now: initialNow };
  const history: Tick[] = [];
  return {
    now: () => state.now,
    sleep: async (ms: number): Promise<void> => {
      history.push({ kind: "sleep", ms });
      state.now += ms;
    },
    history,
    recordSupplier: () => history.push({ kind: "supplier" }),
  };
}

describe("pollForImageAuthenticationKeyword", () => {
  it("returns immediately when the first supplier call yields a match", async () => {
    const clock = createFakeClock(0);
    const keyword = await pollForImageAuthenticationKeyword(
      async () => {
        clock.recordSupplier();
        return "みかん + りんご";
      },
      { config, now: clock.now, sleep: clock.sleep },
    );
    expect(keyword).toEqual({ first: "みかん", second: "りんご" });
    expect(clock.history).toEqual([{ kind: "supplier" }]);
  });

  it("polls at the configured interval until a keyword appears", async () => {
    const clock = createFakeClock(0);
    let invocations = 0;
    const keyword = await pollForImageAuthenticationKeyword(
      async () => {
        clock.recordSupplier();
        invocations += 1;
        if (invocations < 3) {
          return null;
        }
        return "ぶどう + いちご";
      },
      { config, now: clock.now, sleep: clock.sleep },
    );
    expect(keyword).toEqual({ first: "ぶどう", second: "いちご" });
    expect(clock.history).toEqual([
      { kind: "supplier" },
      { kind: "sleep", ms: 100 },
      { kind: "supplier" },
      { kind: "sleep", ms: 100 },
      { kind: "supplier" },
    ]);
  });

  it("throws MailRetrievalTimeoutError when the cap is reached", async () => {
    const clock = createFakeClock(0);
    await expect(
      pollForImageAuthenticationKeyword(
        async () => {
          clock.recordSupplier();
          return null;
        },
        { config, now: clock.now, sleep: clock.sleep },
      ),
    ).rejects.toBeInstanceOf(MailRetrievalTimeoutError);
    const supplierCalls = clock.history.filter(
      (tick) => tick.kind === "supplier",
    );
    expect(supplierCalls.length).toBeGreaterThanOrEqual(2);
    const sleeps = clock.history.filter(
      (tick): tick is { kind: "sleep"; ms: number } => tick.kind === "sleep",
    );
    const lastSleep = sleeps[sleeps.length - 1];
    expect(lastSleep?.ms).toBeLessThanOrEqual(config.pollingIntervalMs);
  });

  it("caps the effective timeout at config.maxTimeoutMs even when a longer window is requested", async () => {
    const clock = createFakeClock(0);
    await expect(
      pollForImageAuthenticationKeyword(
        async () => null,
        {
          timeoutMs: 10_000,
          config,
          now: clock.now,
          sleep: clock.sleep,
        },
      ),
    ).rejects.toBeInstanceOf(MailRetrievalTimeoutError);
    expect(clock.now()).toBeLessThanOrEqual(
      config.maxTimeoutMs + config.pollingIntervalMs,
    );
  });

  it("shortens the last sleep so it never overruns the deadline", async () => {
    const clock = createFakeClock(0);
    await expect(
      pollForImageAuthenticationKeyword(
        async () => null,
        {
          config: { pollingIntervalMs: 400, maxTimeoutMs: 500 },
          now: clock.now,
          sleep: clock.sleep,
        },
      ),
    ).rejects.toBeInstanceOf(MailRetrievalTimeoutError);
    const sleeps = clock.history.filter(
      (tick): tick is { kind: "sleep"; ms: number } => tick.kind === "sleep",
    );
    expect(sleeps.map((s) => s.ms)).toEqual([400, 100]);
  });
});
