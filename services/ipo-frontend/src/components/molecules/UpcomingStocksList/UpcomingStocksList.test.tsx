import { render, screen } from "@testing-library/react";
import {
  dashboardUpcomingStockSchema,
  type DashboardUpcomingStock,
} from "@ipotto/shared";
import { describe, expect, it } from "vitest";

import { UpcomingStocksList } from "./UpcomingStocksList";

const buildStock = (): DashboardUpcomingStock =>
  dashboardUpcomingStockSchema.parse({
    stock: "stock_abc",
    companyName: "株式会社テスト",
    bookBuildingStartDate: "2026-04-01",
    bookBuildingEndDate: "2026-04-10",
    lotteryDate: "2026-04-15",
  });

describe("UpcomingStocksList", () => {
  it("renders upcoming stock schedules", () => {
    render(<UpcomingStocksList stocks={[buildStock()]} />);
    expect(screen.getByText("株式会社テスト")).toBeDefined();
    expect(screen.getByText(/2026-04-01/)).toBeDefined();
  });

  it("shows placeholder on empty list", () => {
    render(<UpcomingStocksList stocks={[]} />);
    expect(screen.getByText("対象の銘柄はありません。")).toBeDefined();
  });
});
