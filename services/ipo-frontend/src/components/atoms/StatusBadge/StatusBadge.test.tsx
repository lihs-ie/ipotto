import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { StatusBadge } from "./StatusBadge";

describe("StatusBadge", () => {
  it("maps Won to success tone", () => {
    render(<StatusBadge status="Won" />);
    const badge = screen.getByText("Won");
    expect(badge.getAttribute("data-tone")).toBe("success");
  });

  it("maps Failed to danger tone", () => {
    render(<StatusBadge status="Failed" />);
    expect(screen.getByText("Failed").getAttribute("data-tone")).toBe("danger");
  });

  it("maps Eligible to warning tone", () => {
    render(<StatusBadge status="Eligible" />);
    expect(screen.getByText("Eligible").getAttribute("data-tone")).toBe(
      "warning",
    );
  });

  it("renders the provided label when given", () => {
    render(<StatusBadge status="Won" label="当選" />);
    expect(screen.getByText("当選")).toBeDefined();
  });
});
