import { describe, expect, it } from "vitest";

import {
  resolveIpoListBlockActionKind,
  selectPreferredIpoListBlockAction,
  type IpoListBlockSnapshot,
} from "./ipo-list-block-action.js";

function createSnapshot(
  overrides: Partial<IpoListBlockSnapshot>,
): IpoListBlockSnapshot {
  return {
    index: 0,
    hasApplyButton: false,
    hasApplicationDetailButton: false,
    hasStockDetailButton: false,
    hasStockLink: false,
    ...overrides,
  };
}

describe("resolveIpoListBlockActionKind", () => {
  it("prefers the application button over all other actions", () => {
    expect(
      resolveIpoListBlockActionKind(
        createSnapshot({
          hasApplyButton: true,
          hasApplicationDetailButton: true,
          hasStockDetailButton: true,
          hasStockLink: true,
        }),
      ),
    ).toBe("apply");
  });

  it("uses application detail for already-applied entries", () => {
    expect(
      resolveIpoListBlockActionKind(
        createSnapshot({
          hasApplicationDetailButton: true,
          hasStockDetailButton: true,
          hasStockLink: true,
        }),
      ),
    ).toBe("application_detail");
  });

  it("falls back to stock detail and then the stock link", () => {
    expect(
      resolveIpoListBlockActionKind(
        createSnapshot({
          hasStockDetailButton: true,
          hasStockLink: true,
        }),
      ),
    ).toBe("stock_detail");

    expect(
      resolveIpoListBlockActionKind(
        createSnapshot({
          hasStockLink: true,
        }),
      ),
    ).toBe("stock_link");
  });

  it("returns null when a block has no actionable entry point", () => {
    expect(resolveIpoListBlockActionKind(createSnapshot({}))).toBeNull();
  });
});

describe("selectPreferredIpoListBlockAction", () => {
  it("prefers the actionable duplicate with the highest priority", () => {
    expect(
      selectPreferredIpoListBlockAction([
        createSnapshot({
          index: 0,
          hasApplicationDetailButton: true,
          hasStockLink: true,
        }),
        createSnapshot({
          index: 1,
          hasApplyButton: true,
          hasStockDetailButton: true,
          hasStockLink: true,
        }),
      ]),
    ).toEqual({
      index: 1,
      kind: "apply",
    });
  });

  it("keeps DOM order when duplicates have the same priority", () => {
    expect(
      selectPreferredIpoListBlockAction([
        createSnapshot({
          index: 0,
          hasApplicationDetailButton: true,
        }),
        createSnapshot({
          index: 1,
          hasApplicationDetailButton: true,
        }),
      ]),
    ).toEqual({
      index: 0,
      kind: "application_detail",
    });
  });

  it("returns null when no duplicate block is actionable", () => {
    expect(
      selectPreferredIpoListBlockAction([
        createSnapshot({
          index: 0,
        }),
        createSnapshot({
          index: 1,
        }),
      ]),
    ).toBeNull();
  });
});
