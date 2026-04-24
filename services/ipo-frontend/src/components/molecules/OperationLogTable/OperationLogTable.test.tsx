import { render, screen } from "@testing-library/react";
import {
  operationLogSummarySchema,
  type OperationLogSummary,
} from "@ipotto/shared";
import { describe, expect, it } from "vitest";

import { OperationLogTable } from "./OperationLogTable";

const buildItem = (): OperationLogSummary =>
  operationLogSummarySchema.parse({
    identifier: "log_001",
    stock: null,
    eventType: "fetch_stocks",
    serviceName: "ipo-info-fetcher",
    status: "succeeded",
    message: "ok",
    errorDetails: null,
    executedAt: "2026-03-25T10:00:00Z",
  });

describe("OperationLogTable", () => {
  it("renders log rows", () => {
    render(<OperationLogTable items={[buildItem()]} />);
    expect(screen.getByText("fetch_stocks")).toBeDefined();
    expect(screen.getByText("ipo-info-fetcher")).toBeDefined();
    expect(screen.getByText("succeeded")).toBeDefined();
  });

  it("renders empty placeholder when no items", () => {
    render(<OperationLogTable items={[]} />);
    expect(screen.getByText("ログはありません。")).toBeDefined();
  });
});
