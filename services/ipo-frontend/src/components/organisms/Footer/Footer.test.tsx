import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { Footer } from "./Footer";

describe("Footer", () => {
  it("renders the provided copyright year", () => {
    render(<Footer year={2026} />);
    expect(screen.getByText(/© 2026 IPOtto/)).toBeDefined();
  });

  it("defaults to the current year when no prop is given", () => {
    render(<Footer />);
    const currentYear = new Date().getFullYear();
    expect(screen.getByText(new RegExp(`© ${currentYear}`))).toBeDefined();
  });
});
