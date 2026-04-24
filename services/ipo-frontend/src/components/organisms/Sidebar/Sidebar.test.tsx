import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { Sidebar } from "./Sidebar";

describe("Sidebar", () => {
  it("renders all 6 navigation links", () => {
    render(<Sidebar currentPath="/" />);
    for (const label of [
      "ダッシュボード",
      "IPO銘柄",
      "除外リスト",
      "通知設定",
      "証券口座",
      "操作ログ",
    ]) {
      expect(screen.getByRole("link", { name: label })).toBeDefined();
    }
  });

  it("marks the dashboard link as active on /", () => {
    render(<Sidebar currentPath="/" />);
    const link = screen.getByRole("link", { name: "ダッシュボード" });
    expect(link.getAttribute("data-active")).toBe("true");
  });

  it("marks the stocks link as active on /stocks/{id} routes too", () => {
    render(<Sidebar currentPath="/stocks/abc_123" />);
    const link = screen.getByRole("link", { name: "IPO銘柄" });
    expect(link.getAttribute("data-active")).toBe("true");
  });

  it("only activates one link for any given path", () => {
    render(<Sidebar currentPath="/logs" />);
    const active = screen.getByRole("link", { name: "操作ログ" });
    expect(active.getAttribute("data-active")).toBe("true");
    const dashboard = screen.getByRole("link", { name: "ダッシュボード" });
    expect(dashboard.getAttribute("data-active")).toBe("false");
  });
});
