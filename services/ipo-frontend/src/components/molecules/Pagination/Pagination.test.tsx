import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { Pagination } from "./Pagination";

describe("Pagination", () => {
  it("disables the next button when hasMore is false", () => {
    render(<Pagination hasMore={false} onNext={() => undefined} />);
    const button = screen.getByRole("button", { name: "次へ" });
    expect(button.hasAttribute("disabled")).toBe(true);
  });

  it("calls onNext when enabled and clicked", () => {
    const onNext = vi.fn();
    render(<Pagination hasMore={true} onNext={onNext} />);
    fireEvent.click(screen.getByRole("button", { name: "次へ" }));
    expect(onNext).toHaveBeenCalledTimes(1);
  });

  it("disables the next button while loading", () => {
    render(<Pagination hasMore={true} onNext={() => undefined} loading />);
    expect(
      screen.getByRole("button", { name: "次へ" }).hasAttribute("disabled"),
    ).toBe(true);
  });
});
