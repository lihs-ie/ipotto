import { render, screen } from "@testing-library/react";
import {
  dashboardRecentActivitySchema,
  type DashboardRecentActivity,
} from "@ipotto/shared";
import { describe, expect, it } from "vitest";

import { RecentActivitiesList } from "./RecentActivitiesList";

const buildActivity = (): DashboardRecentActivity =>
  dashboardRecentActivitySchema.parse({
    stock: "stock_abc",
    companyName: "テスト株式会社",
    securitiesCompany: "楽天証券",
    eventType: "ApplicationCompleted",
    occurredAt: "2026-03-25T10:00:00Z",
  });

describe("RecentActivitiesList", () => {
  it("renders each activity row", () => {
    render(<RecentActivitiesList activities={[buildActivity()]} />);
    expect(screen.getByText("テスト株式会社")).toBeDefined();
    expect(screen.getByText("ApplicationCompleted")).toBeDefined();
    expect(screen.getByText("楽天証券")).toBeDefined();
  });

  it("shows a placeholder when activities are empty", () => {
    render(<RecentActivitiesList activities={[]} />);
    expect(screen.getByText("活動履歴はありません。")).toBeDefined();
  });
});
