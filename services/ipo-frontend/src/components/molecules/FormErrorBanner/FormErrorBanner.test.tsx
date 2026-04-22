import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { FormErrorBanner } from "./FormErrorBanner";

describe("FormErrorBanner", () => {
  it("returns null when error is null", () => {
    const { container } = render(<FormErrorBanner error={null} />);
    expect(container.firstChild).toBeNull();
  });

  it("renders HTTP error code + message", () => {
    render(
      <FormErrorBanner
        error={{
          kind: "http",
          status: 409,
          body: {
            error: {
              code: "DUPLICATE_CHANNEL_TYPE",
              message: "重複しています",
            },
          },
        }}
      />,
    );
    expect(
      screen.getByText(/DUPLICATE_CHANNEL_TYPE.*重複しています/),
    ).toBeDefined();
  });

  it("lists validation details when present", () => {
    render(
      <FormErrorBanner
        error={{
          kind: "http",
          status: 400,
          body: {
            error: {
              code: "VALIDATION_ERROR",
              message: "入力値が不正です",
              details: [{ field: "mailAddress", message: "形式不正" }],
            },
          },
        }}
      />,
    );
    expect(screen.getByText(/mailAddress: 形式不正/)).toBeDefined();
  });

  it("renders network error message", () => {
    render(
      <FormErrorBanner
        error={{ kind: "network", message: "offline" }}
      />,
    );
    expect(screen.getByText(/ネットワークエラー: offline/)).toBeDefined();
  });
});
