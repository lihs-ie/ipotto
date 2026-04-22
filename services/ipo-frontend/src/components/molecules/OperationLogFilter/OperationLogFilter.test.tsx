import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { OperationLogFilter } from "./OperationLogFilter";

describe("OperationLogFilter", () => {
  it("calls onChange with the selected event type", () => {
    const onChange = vi.fn();
    render(
      <OperationLogFilter value={{ eventType: null }} onChange={onChange} />,
    );
    fireEvent.change(screen.getByRole("combobox"), {
      target: { value: "fetch_stocks" },
    });
    expect(onChange).toHaveBeenCalledWith({ eventType: "fetch_stocks" });
  });

  it("calls onChange with null when 'すべて' selected", () => {
    const onChange = vi.fn();
    render(
      <OperationLogFilter
        value={{ eventType: "fetch_stocks" }}
        onChange={onChange}
      />,
    );
    fireEvent.change(screen.getByRole("combobox"), { target: { value: "" } });
    expect(onChange).toHaveBeenCalledWith({ eventType: null });
  });
});
