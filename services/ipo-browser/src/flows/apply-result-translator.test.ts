import { describe, expect, it } from "vitest";

import { translateApplyResultText } from "./apply-result-translator.js";

describe("translateApplyResultText", () => {
  it("maps '受け付けました' to success", () => {
    expect(translateApplyResultText("申込を受け付けました")).toEqual({
      status: "success",
    });
  });

  it("maps '完了' to success", () => {
    expect(translateApplyResultText("申込完了")).toEqual({ status: "success" });
  });

  it("maps '既に申込済み' to already_applied", () => {
    expect(translateApplyResultText("既に申込済みです")).toEqual({
      status: "already_applied",
    });
  });

  it("maps '残高' + '不足' to insufficient_balance", () => {
    expect(translateApplyResultText("残高が不足しています")).toEqual({
      status: "insufficient_balance",
    });
  });

  it("falls back to failure with raw text for unknown messages", () => {
    expect(translateApplyResultText("システムエラー")).toEqual({
      status: "failure",
      reason: "システムエラー",
    });
  });

  it("trims surrounding whitespace before matching", () => {
    expect(translateApplyResultText("   受け付けました   ")).toEqual({
      status: "success",
    });
  });
});
