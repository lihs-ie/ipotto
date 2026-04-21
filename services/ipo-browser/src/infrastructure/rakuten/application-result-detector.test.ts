import { describe, expect, it } from "vitest";

import {
  detectApplicationResultFromText,
  normalizeRakutenPageText,
} from "./application-result-detector.js";

describe("application-result-detector", () => {
  it("normalizes Rakuten page text", () => {
    expect(normalizeRakutenPageText("  a \n  b\t c  ")).toBe("a b c");
  });

  it("detects success", () => {
    expect(
      detectApplicationResultFromText(
        "ブックビルディングの申込を受け付けました",
        "failed",
      ),
    ).toEqual({ status: "success" });
  });

  it("detects already applied", () => {
    expect(
      detectApplicationResultFromText("ブックビルディング申込済", "failed"),
    ).toEqual({ status: "already_applied" });
  });

  it("detects insufficient balance", () => {
    expect(
      detectApplicationResultFromText("買付可能額が不足しています", "failed"),
    ).toEqual({ status: "insufficient_balance" });
  });

  it("falls back to a failed result", () => {
    expect(
      detectApplicationResultFromText("原因不明の画面", "selector missing"),
    ).toEqual({
      status: "failure",
      reason: "selector missing",
      category: "application",
    });
  });
});
