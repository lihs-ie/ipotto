import { fireEvent, render, screen } from "@testing-library/react";
import {
  exclusionSummarySchema,
  type ExclusionSummary,
} from "@ipotto/shared";
import { describe, expect, it, vi } from "vitest";

import { ExclusionTable } from "./ExclusionTable";

const buildExclusion = (): ExclusionSummary =>
  exclusionSummarySchema.parse({
    identifier: "excl_abc",
    companyName: "除外対象株式会社",
    reason: "テスト",
    registeredAt: "2026-03-25T10:00:00Z",
  });

describe("ExclusionTable", () => {
  it("renders exclusions", () => {
    render(
      <ExclusionTable items={[buildExclusion()]} onDelete={() => undefined} />,
    );
    expect(screen.getByText("除外対象株式会社")).toBeDefined();
    expect(screen.getByText("テスト")).toBeDefined();
  });

  it("renders empty placeholder", () => {
    render(<ExclusionTable items={[]} onDelete={() => undefined} />);
    expect(
      screen.getByText("除外対象はまだ登録されていません。"),
    ).toBeDefined();
  });

  it("fires onDelete when a row's button is clicked", () => {
    const onDelete = vi.fn();
    render(<ExclusionTable items={[buildExclusion()]} onDelete={onDelete} />);
    fireEvent.click(screen.getByRole("button", { name: "削除" }));
    expect(onDelete).toHaveBeenCalledWith("excl_abc");
  });
});
