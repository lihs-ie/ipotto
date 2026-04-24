import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { SystemStatusCard } from "./SystemStatusCard";

describe("SystemStatusCard", () => {
  it("renders the next job time and account statuses", () => {
    render(
      <SystemStatusCard
        nextJobScheduledAt="2026-04-15T09:00:00Z"
        accounts={[
          {
            securitiesCompany: "楽天証券",
            connectionStatus: "healthy",
            lastTestedAt: null,
          },
        ]}
      />,
    );
    expect(screen.getByText("楽天証券")).toBeDefined();
    expect(screen.getByText("healthy")).toBeDefined();
  });
});
