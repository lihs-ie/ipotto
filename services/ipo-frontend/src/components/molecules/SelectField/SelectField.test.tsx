import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { SelectField } from "./SelectField";

describe("SelectField", () => {
  it("renders options and the current value", () => {
    render(
      <SelectField
        label="証券会社"
        value="Rakuten"
        onChange={() => undefined}
        options={[{ label: "楽天証券", value: "Rakuten" }]}
      />,
    );
    expect(screen.getByText("証券会社")).toBeDefined();
    const select = screen.getByRole("combobox") as HTMLSelectElement;
    expect(select.value).toBe("Rakuten");
  });

  it("fires onChange with the next value", () => {
    const onChange = vi.fn();
    render(
      <SelectField
        label="種別"
        value="a"
        onChange={onChange}
        options={[
          { label: "A", value: "a" },
          { label: "B", value: "b" },
        ]}
      />,
    );
    fireEvent.change(screen.getByRole("combobox"), { target: { value: "b" } });
    expect(onChange).toHaveBeenCalledWith("b");
  });
});
