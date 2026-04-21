import { describe, expect, it } from "vitest";

import { chooseImageIndices } from "./matcher.js";

const fruits = [
  { altText: "みかん" },
  { altText: "りんご" },
  { altText: "ぶどう" },
  { altText: "いちご" },
  { altText: "ばなな" },
  { altText: "すいか" },
  { altText: "めろん" },
];

describe("chooseImageIndices", () => {
  it("returns both indices in keyword order on a clean match", () => {
    const outcome = chooseImageIndices(
      { first: "みかん", second: "りんご" },
      fruits,
    );
    expect(outcome).toEqual({ status: "matched", indices: [0, 1] });
  });

  it("preserves keyword order even when the second keyword appears earlier", () => {
    const outcome = chooseImageIndices(
      { first: "ばなな", second: "みかん" },
      fruits,
    );
    expect(outcome).toEqual({ status: "matched", indices: [4, 0] });
  });

  it("trims surrounding whitespace and collapses fullwidth spaces on both sides", () => {
    const buttons = [
      { altText: " みかん " },
      { altText: "りんご　" },
    ];
    const outcome = chooseImageIndices(
      { first: "みかん", second: "　りんご" },
      buttons,
    );
    expect(outcome).toEqual({ status: "matched", indices: [0, 1] });
  });

  it("reports the missing keyword when one side has no match", () => {
    const outcome = chooseImageIndices(
      { first: "みかん", second: "存在しない果物" },
      fruits,
    );
    expect(outcome).toEqual({
      status: "missing_keyword",
      keyword: "存在しない果物",
    });
  });

  it("short-circuits on the first missing keyword without evaluating the second", () => {
    const outcome = chooseImageIndices(
      { first: "見つからない", second: "みかん" },
      fruits,
    );
    expect(outcome).toEqual({
      status: "missing_keyword",
      keyword: "見つからない",
    });
  });

  it("flags ambiguous matches so the caller can defer to manual fallback", () => {
    const buttons = [
      { altText: "みかん" },
      { altText: "みかん" },
      { altText: "りんご" },
    ];
    const outcome = chooseImageIndices(
      { first: "みかん", second: "りんご" },
      buttons,
    );
    expect(outcome).toEqual({
      status: "ambiguous_keyword",
      keyword: "みかん",
      indices: [0, 1],
    });
  });

  it("returns matched even when the two keywords resolve to the same button index", () => {
    // Rakuten never emits identical keywords but keep the behaviour
    // deterministic in case it ever does: both keywords resolving to
    // button 0 should surface indices [0, 0] and let the caller decide.
    const buttons = [{ altText: "みかん" }, { altText: "りんご" }];
    const outcome = chooseImageIndices(
      { first: "みかん", second: "みかん" },
      buttons,
    );
    expect(outcome).toEqual({ status: "matched", indices: [0, 0] });
  });

  it("treats an empty button list as a missing keyword", () => {
    const outcome = chooseImageIndices(
      { first: "みかん", second: "りんご" },
      [],
    );
    expect(outcome).toEqual({ status: "missing_keyword", keyword: "みかん" });
  });
});
