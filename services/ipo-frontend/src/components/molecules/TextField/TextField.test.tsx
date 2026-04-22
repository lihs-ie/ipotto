import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { TextField } from "./TextField";

describe("TextField", () => {
  it("renders the label and current value", () => {
    render(
      <TextField
        label="会社名"
        value="テスト株式会社"
        onChange={() => undefined}
      />,
    );
    expect(screen.getByText("会社名")).toBeDefined();
    expect(
      (screen.getByRole("textbox") as HTMLInputElement).value,
    ).toBe("テスト株式会社");
  });

  it("fires onChange with the next value", () => {
    const onChange = vi.fn();
    render(<TextField label="会社名" value="" onChange={onChange} />);
    fireEvent.change(screen.getByRole("textbox"), {
      target: { value: "next" },
    });
    expect(onChange).toHaveBeenCalledWith("next");
  });

  it("shows the error text when provided", () => {
    render(
      <TextField
        label="会社名"
        value=""
        onChange={() => undefined}
        error="必須項目です"
      />,
    );
    expect(screen.getByText("必須項目です")).toBeDefined();
  });
});
