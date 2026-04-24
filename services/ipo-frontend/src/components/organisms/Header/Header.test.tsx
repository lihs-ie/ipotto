import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { Header, resolveBreadcrumbs } from "./Header";

describe("Header", () => {
  const defaultBreadcrumbs = [{ label: "ダッシュボード", path: "/" }];

  it("renders the user email when provided", () => {
    render(
      <Header
        userEmail="user@example.com"
        onSignOut={async () => undefined}
        breadcrumbs={defaultBreadcrumbs}
      />,
    );
    expect(screen.getByText("user@example.com")).toBeDefined();
  });

  it("invokes onSignOut when logout button is pressed", () => {
    const onSignOut = vi.fn().mockResolvedValue(undefined);
    render(
      <Header
        userEmail="user@example.com"
        onSignOut={onSignOut}
        breadcrumbs={defaultBreadcrumbs}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /ログアウト/ }));
    expect(onSignOut).toHaveBeenCalledTimes(1);
  });

  it("omits the email line when userEmail is null", () => {
    render(
      <Header
        userEmail={null}
        onSignOut={async () => undefined}
        breadcrumbs={defaultBreadcrumbs}
      />,
    );
    expect(screen.queryByText(/@/)).toBeNull();
  });

  it("renders breadcrumbs with the active segment styled", () => {
    render(
      <Header
        userEmail={null}
        onSignOut={async () => undefined}
        breadcrumbs={[
          { label: "IPO銘柄", path: "/stocks" },
        ]}
      />,
    );
    expect(screen.getByText("IPO銘柄")).toBeDefined();
  });

  it("renders breadcrumb separators for multiple segments", () => {
    render(
      <Header
        userEmail={null}
        onSignOut={async () => undefined}
        breadcrumbs={[
          { label: "通知設定", path: "/notifications/settings" },
        ]}
      />,
    );
    expect(screen.getByText("通知設定")).toBeDefined();
  });

  it("renders the breadcrumb navigation landmark", () => {
    render(
      <Header
        userEmail={null}
        onSignOut={async () => undefined}
        breadcrumbs={defaultBreadcrumbs}
      />,
    );
    expect(screen.getByRole("navigation", { name: "パンくずリスト" })).toBeDefined();
  });
});

describe("resolveBreadcrumbs", () => {
  it("resolves root path to dashboard", () => {
    expect(resolveBreadcrumbs("/")).toEqual([
      { label: "ダッシュボード", path: "/" },
    ]);
  });

  it("resolves known paths to their labels", () => {
    expect(resolveBreadcrumbs("/stocks")).toEqual([
      { label: "IPO銘柄", path: "/stocks" },
    ]);
  });

  it("resolves nested known paths", () => {
    expect(resolveBreadcrumbs("/notifications/settings")).toEqual([
      { label: "通知設定", path: "/notifications/settings" },
    ]);
  });

  it("falls back to dashboard for unknown paths", () => {
    expect(resolveBreadcrumbs("/unknown/page")).toEqual([
      { label: "ダッシュボード", path: "/" },
    ]);
  });

  it("resolves sub-paths with known parent segments", () => {
    expect(resolveBreadcrumbs("/stocks/detail")).toEqual([
      { label: "IPO銘柄", path: "/stocks" },
    ]);
  });
});
