import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { Button } from "./Button";

describe("Button", () => {
  it("renders the provided label", () => {
    render(<Button label="登録" />);
    expect(screen.getByRole("button", { name: "登録" })).toBeDefined();
  });

  it("invokes onClick when pressed", () => {
    const handleClick = vi.fn();
    render(<Button label="送信" onClick={handleClick} />);
    fireEvent.click(screen.getByRole("button"));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it("does not invoke onClick while disabled", () => {
    const handleClick = vi.fn();
    render(<Button label="送信" onClick={handleClick} disabled />);
    fireEvent.click(screen.getByRole("button"));
    expect(handleClick).not.toHaveBeenCalled();
  });

  it("applies the variant attribute", () => {
    render(<Button label="キャンセル" variant="secondary" />);
    expect(screen.getByRole("button").getAttribute("data-variant")).toBe(
      "secondary",
    );
  });
});
