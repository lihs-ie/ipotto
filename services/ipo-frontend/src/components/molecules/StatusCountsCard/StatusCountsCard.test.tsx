import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { StatusCountsCard } from "./StatusCountsCard";

describe("StatusCountsCard", () => {
  it("renders each status with its count", () => {
    render(<StatusCountsCard counts={{ Fetched: 2, Eligible: 5, Won: 1 }} />);
    expect(screen.getByText("2")).toBeDefined();
    expect(screen.getByText("5")).toBeDefined();
    expect(screen.getByText("1")).toBeDefined();
  });

  it("defaults to 0 for missing statuses", () => {
    render(<StatusCountsCard counts={{}} />);
    const zeros = screen.getAllByText("0");
    expect(zeros.length).toBe(11);
  });
});
