import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ExclusionForm } from "./ExclusionForm";

describe("ExclusionForm", () => {
  it("validates required fields via Zod", async () => {
    const onSubmit = vi.fn().mockResolvedValue(undefined);
    render(<ExclusionForm pending={false} onSubmit={onSubmit} />);
    fireEvent.submit(screen.getByRole("button", { name: "登録" }).closest("form")!);
    await waitFor(() => {
      expect(onSubmit).not.toHaveBeenCalled();
    });
  });

  it("calls onSubmit with validated payload", async () => {
    const onSubmit = vi.fn().mockResolvedValue(undefined);
    render(<ExclusionForm pending={false} onSubmit={onSubmit} />);
    const inputs = screen.getAllByRole("textbox");
    fireEvent.change(inputs[0]!, { target: { value: "除外株式会社" } });
    fireEvent.change(inputs[1]!, { target: { value: "テスト理由" } });
    fireEvent.submit(
      screen.getByRole("button", { name: "登録" }).closest("form")!,
    );
    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledWith({
        companyName: "除外株式会社",
        reason: "テスト理由",
      });
    });
  });

  it("disables the submit button while pending", () => {
    render(<ExclusionForm pending={true} onSubmit={vi.fn()} />);
    expect(
      screen.getByRole("button", { name: "登録" }).hasAttribute("disabled"),
    ).toBe(true);
  });
});
