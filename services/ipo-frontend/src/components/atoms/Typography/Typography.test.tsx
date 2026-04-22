import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { Typography } from "./Typography";

describe("Typography", () => {
  it("renders h1 when variant is h1", () => {
    render(<Typography variant="h1">見出し</Typography>);
    const heading = screen.getByRole("heading", { level: 1 });
    expect(heading.textContent).toBe("見出し");
  });

  it("renders h2 when variant is h2", () => {
    render(<Typography variant="h2">サブ見出し</Typography>);
    expect(screen.getByRole("heading", { level: 2 }).textContent).toBe(
      "サブ見出し",
    );
  });

  it("renders paragraph for body variant", () => {
    render(<Typography variant="body">本文</Typography>);
    expect(screen.getByText("本文").tagName).toBe("P");
  });
});
