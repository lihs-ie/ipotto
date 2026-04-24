import { describe, expect, it } from "vitest";

import {
  applicationStatusSchema,
  channelTypeSchema,
  fetchOriginSchema,
  lotteryResultSchema,
  marketSchema,
  notificationEventTypeSchema,
  operationEventTypeSchema,
  operationStatusSchema,
  securitiesCompanySchema,
  stockStatusSchema,
} from "./enums";

describe("enum schemas", () => {
  it("stockStatus accepts every canonical Rust variant", () => {
    const variants = [
      "Fetched",
      "Eligible",
      "Applied",
      "Won",
      "Lost",
      "Alternate",
      "Purchased",
      "Declined",
      "Sold",
      "Excluded",
      "Failed",
    ];
    for (const variant of variants) {
      expect(stockStatusSchema.parse(variant)).toBe(variant);
    }
  });

  it("stockStatus rejects unknown value", () => {
    expect(() => stockStatusSchema.parse("PendingReview")).toThrow();
  });

  it("market accepts Prime/Standard/Growth", () => {
    for (const variant of ["Prime", "Standard", "Growth"]) {
      expect(marketSchema.parse(variant)).toBe(variant);
    }
  });

  it("market rejects legacy Japanese labels", () => {
    expect(() => marketSchema.parse("グロース")).toThrow();
  });

  it("applicationStatus matches Rust Pending/Applied/ResultChecked", () => {
    for (const variant of ["Pending", "Applied", "ResultChecked"]) {
      expect(applicationStatusSchema.parse(variant)).toBe(variant);
    }
  });

  it("applicationStatus rejects lottery outcome values", () => {
    expect(() => applicationStatusSchema.parse("Won")).toThrow();
  });

  it("lotteryResult accepts Won/Lost/Alternate", () => {
    for (const variant of ["Won", "Lost", "Alternate"]) {
      expect(lotteryResultSchema.parse(variant)).toBe(variant);
    }
  });

  it("channelType accepts LINE/Email/Slack", () => {
    for (const variant of ["LINE", "Email", "Slack"]) {
      expect(channelTypeSchema.parse(variant)).toBe(variant);
    }
  });

  it("fetchOrigin matches Rust ExternalSite/SecuritiesSite", () => {
    expect(fetchOriginSchema.parse("ExternalSite")).toBe("ExternalSite");
    expect(fetchOriginSchema.parse("SecuritiesSite")).toBe("SecuritiesSite");
  });

  it("notificationEventType has 5 canonical values", () => {
    for (const variant of [
      "ApplicationCompleted",
      "LotteryResultWon",
      "LotteryResultLost",
      "OperationError",
      "StockUpdated",
    ]) {
      expect(notificationEventTypeSchema.parse(variant)).toBe(variant);
    }
  });

  it("operationEventType uses snake_case", () => {
    for (const variant of [
      "fetch_stocks",
      "apply_lottery",
      "check_lottery_result",
      "notification_dispatch",
      "connection_test",
      "other",
    ]) {
      expect(operationEventTypeSchema.parse(variant)).toBe(variant);
    }
  });

  it("operationEventType rejects CamelCase variant", () => {
    expect(() => operationEventTypeSchema.parse("FetchStocks")).toThrow();
  });

  it("operationStatus accepts succeeded/failed", () => {
    expect(operationStatusSchema.parse("succeeded")).toBe("succeeded");
    expect(operationStatusSchema.parse("failed")).toBe("failed");
  });

  it("securitiesCompany only accepts Rakuten for Phase 1 MVP", () => {
    expect(securitiesCompanySchema.parse("Rakuten")).toBe("Rakuten");
    expect(() => securitiesCompanySchema.parse("SBI")).toThrow();
  });
});
