import { fireEvent, render, screen } from "@testing-library/react";
import {
  ipoStockSummarySchema,
  type IpoStockSummary,
} from "@ipotto/shared";
import { describe, expect, it, vi } from "vitest";

import { StockTable } from "./StockTable";

const buildStock = (): IpoStockSummary =>
  ipoStockSummarySchema.parse({
    identifier: "stock_abc123",
    companyName: "テスト株式会社",
    tickerSymbol: "1234",
    market: "Growth",
    industry: "情報・通信業",
    bookBuildingStartDate: "2026-04-01",
    bookBuildingEndDate: "2026-04-10",
    lotteryDate: "2026-04-15",
    listingDate: "2026-04-25",
    priceRangeMin: 1200,
    priceRangeMax: 1500,
    offerPrice: 1400,
    leadUnderwriter: "楽天証券",
    numberOfOfferedShares: 100_000,
    status: "Eligible",
  });

describe("StockTable", () => {
  it("renders a row per stock", () => {
    render(<StockTable stocks={[buildStock()]} onRowClick={() => undefined} />);
    expect(screen.getByText("テスト株式会社")).toBeDefined();
    expect(screen.getByText(/1,200 〜 1,500円/)).toBeDefined();
  });

  it("invokes onRowClick with the stock identifier", () => {
    const onRowClick = vi.fn();
    render(<StockTable stocks={[buildStock()]} onRowClick={onRowClick} />);
    fireEvent.click(screen.getByText("テスト株式会社"));
    expect(onRowClick).toHaveBeenCalledWith("stock_abc123");
  });

  it("renders empty placeholder when no stocks", () => {
    render(<StockTable stocks={[]} onRowClick={() => undefined} />);
    expect(screen.getByText("対象の銘柄はありません。")).toBeDefined();
  });
});
