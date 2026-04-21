import { describe, expect, it } from "vitest";

import { generateUlid } from "./generate-ulid.js";

describe("generateUlid", () => {
  it("returns a valid ULID", () => {
    expect(generateUlid()).toMatch(/^[0-9A-HJKMNP-TV-Z]{26}$/u);
  });

  it("returns monotonic values when called sequentially", () => {
    const first = generateUlid();
    const second = generateUlid();

    expect(first < second).toBe(true);
  });
});
