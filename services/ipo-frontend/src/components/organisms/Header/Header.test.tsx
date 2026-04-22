import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { Header } from "./Header";

describe("Header", () => {
  it("renders the user email when provided", () => {
    render(
      <Header userEmail="user@example.com" onSignOut={async () => undefined} />,
    );
    expect(screen.getByText("user@example.com")).toBeDefined();
  });

  it("invokes onSignOut when logout button is pressed", () => {
    const onSignOut = vi.fn().mockResolvedValue(undefined);
    render(<Header userEmail="user@example.com" onSignOut={onSignOut} />);
    fireEvent.click(screen.getByRole("button", { name: /ログアウト/ }));
    expect(onSignOut).toHaveBeenCalledTimes(1);
  });

  it("omits the email line when userEmail is null", () => {
    render(<Header userEmail={null} onSignOut={async () => undefined} />);
    expect(screen.queryByText(/@/)).toBeNull();
  });
});
