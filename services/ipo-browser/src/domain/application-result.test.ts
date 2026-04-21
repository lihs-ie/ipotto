import { describe, expect, it } from "vitest";

import {
  createAlreadyAppliedResult,
  createFailedApplicationResult,
  createInsufficientBalanceResult,
  createSuccessfulApplicationResult,
} from "./application-result.js";

describe("application-result helpers", () => {
  it("creates a successful result", () => {
    expect(createSuccessfulApplicationResult()).toEqual({ status: "success" });
  });

  it("creates a failed result with the default category", () => {
    expect(createFailedApplicationResult("selector missing")).toEqual({
      status: "failure",
      reason: "selector missing",
      category: "application",
    });
  });

  it("creates a failed result with an explicit category", () => {
    expect(
      createFailedApplicationResult("mail timeout", "mail_retrieval"),
    ).toEqual({
      status: "failure",
      reason: "mail timeout",
      category: "mail_retrieval",
    });
  });

  it("creates already applied and insufficient balance results", () => {
    expect(createAlreadyAppliedResult()).toEqual({ status: "already_applied" });
    expect(createInsufficientBalanceResult()).toEqual({
      status: "insufficient_balance",
    });
  });
});
