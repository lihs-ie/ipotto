import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { StockStatusFilter } from "./StockStatusFilter";

describe("StockStatusFilter", () => {
  it("marks 'すべて' as active when value is null", () => {
    render(<StockStatusFilter value={null} onChange={() => undefined} />);
    expect(
      screen.getByRole("button", { name: "すべて" }).getAttribute("data-active"),
    ).toBe("true");
  });

  it("fires onChange with the clicked status", () => {
    const onChange = vi.fn();
    render(<StockStatusFilter value={null} onChange={onChange} />);
    fireEvent.click(screen.getByRole("button", { name: "Applied" }));
    expect(onChange).toHaveBeenCalledWith("Applied");
  });

  it("fires onChange with null when 'すべて' is selected", () => {
    const onChange = vi.fn();
    render(<StockStatusFilter value="Applied" onChange={onChange} />);
    fireEvent.click(screen.getByRole("button", { name: "すべて" }));
    expect(onChange).toHaveBeenCalledWith(null);
  });
});
