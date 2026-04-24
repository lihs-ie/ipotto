import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { DashboardClient } from "./dashboard-client";
import { err, ok } from "@/lib/api/result";
import { type IpoApi } from "@/lib/api/endpoints";

let mockApi: Partial<IpoApi> = {};

vi.mock("@/app/client-providers", () => ({
  useIpoApi: () => mockApi,
}));

const summary = {
  statusCounts: { Fetched: 2, Eligible: 5 },
  recentActivities: [],
  upcomingStocks: [],
  systemStatus: {
    nextJobScheduledAt: "2026-04-15T09:00:00Z",
    accounts: [],
  },
};

describe("DashboardClient", () => {
  it("renders the dashboard heading", () => {
    mockApi = {
      getDashboardSummary: vi.fn().mockResolvedValue(ok(summary)),
    };
    render(<DashboardClient />);
    expect(
      screen.getByRole("heading", { level: 1 }).textContent,
    ).toContain("ダッシュボード");
  });

  it("renders the overview on successful fetch", async () => {
    mockApi = {
      getDashboardSummary: vi.fn().mockResolvedValue(ok(summary)),
    };
    render(<DashboardClient />);
    await waitFor(() =>
      expect(screen.getByText("ステータス別件数")).toBeDefined(),
    );
  });

  it("renders an error message when the fetch fails", async () => {
    mockApi = {
      getDashboardSummary: vi
        .fn()
        .mockResolvedValue(err({ kind: "network", message: "offline" })),
    };
    render(<DashboardClient />);
    await waitFor(() =>
      expect(screen.getByText(/読み込みに失敗しました/)).toBeDefined(),
    );
  });
});
