import type { ApplicationResult } from "../../domain/application-result.js";
import {
  createAlreadyAppliedResult,
  createFailedApplicationResult,
  createInsufficientBalanceResult,
  createSuccessfulApplicationResult,
} from "../../domain/application-result.js";

/**
 * Normalizes Rakuten page text for string-based result detection.
 */
export function normalizeRakutenPageText(pageText: string): string {
  return pageText.replace(/\s+/gu, " ").trim();
}

/**
 * Detects an IPO application result from Rakuten page text.
 */
export function detectApplicationResultFromText(
  pageText: string,
  failureReason: string,
): ApplicationResult {
  const normalizedPageText = normalizeRakutenPageText(pageText);

  if (normalizedPageText.includes("ブックビルディングの申込を受け付けました")) {
    return createSuccessfulApplicationResult();
  }

  if (
    normalizedPageText.includes("ブックビルディング申込済") ||
    normalizedPageText.includes("申込詳細") ||
    normalizedPageText.includes("既に申込済み") ||
    normalizedPageText.includes("すでに申込済み")
  ) {
    return createAlreadyAppliedResult();
  }

  if (
    normalizedPageText.includes("買付可能額が不足") ||
    normalizedPageText.includes("不足金") ||
    normalizedPageText.includes("残高不足")
  ) {
    return createInsufficientBalanceResult();
  }

  return createFailedApplicationResult(failureReason);
}
