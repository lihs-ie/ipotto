import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { Input } from "./Input";

describe("Input", () => {
  it("forwards the current value", () => {
    render(
      <Input value="hello" onChange={() => undefined} placeholder="placeholder" />,
    );
    const input = screen.getByPlaceholderText("placeholder") as HTMLInputElement;
    expect(input.value).toBe("hello");
  });

  it("calls onChange with the next value", () => {
    const handleChange = vi.fn();
    render(
      <Input value="" onChange={handleChange} placeholder="placeholder" />,
    );
    const input = screen.getByPlaceholderText("placeholder");
    fireEvent.change(input, { target: { value: "next" } });
    expect(handleChange).toHaveBeenCalledWith("next");
  });
});
