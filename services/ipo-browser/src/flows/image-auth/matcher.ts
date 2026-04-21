// Phase 3 Sprint 6 — pure logic that resolves Rakuten's image-auth
// keywords against the alt-attribute list collected from the DOM.
//
// Input shape mirrors the domain ACL contract: two keywords emitted by
// the mail-driven OTP reader, each of which must match exactly one
// alt-labelled button. The authentication screen always asks for the
// keywords in order, so the returned indices preserve the order in
// which the operator must click the buttons.

export type ImageAuthenticationKeywords = {
  readonly first: string;
  readonly second: string;
};

export type ImageAuthButton = {
  readonly altText: string;
};

export type MatcherOutcome =
  | {
      readonly status: "matched";
      readonly indices: readonly [number, number];
    }
  | {
      readonly status: "missing_keyword";
      readonly keyword: string;
    }
  | {
      readonly status: "ambiguous_keyword";
      readonly keyword: string;
      readonly indices: readonly number[];
    };

const WHITESPACE_PATTERN = /\s+/g;

/**
 * Resolves the two keywords to their button indices. The comparison is
 * exact match on the alt attribute after trimming / collapsing
 * whitespace so Rakuten's fullwidth vs halfwidth space variations do
 * not defeat the match. Each keyword is expected to hit exactly one
 * button; ambiguity (duplicate alt text) or absence is surfaced so the
 * caller can choose to fall back to the manual flow without guessing.
 */
export function chooseImageIndices(
  keywords: ImageAuthenticationKeywords,
  buttons: readonly ImageAuthButton[],
): MatcherOutcome {
  const firstOutcome = resolveKeyword(keywords.first, buttons);
  if (firstOutcome.status !== "matched") {
    return firstOutcome;
  }
  const secondOutcome = resolveKeyword(keywords.second, buttons);
  if (secondOutcome.status !== "matched") {
    return secondOutcome;
  }
  return {
    status: "matched",
    indices: [firstOutcome.index, secondOutcome.index],
  };
}

type KeywordResolution =
  | { readonly status: "matched"; readonly index: number }
  | { readonly status: "missing_keyword"; readonly keyword: string }
  | {
      readonly status: "ambiguous_keyword";
      readonly keyword: string;
      readonly indices: readonly number[];
    };

function resolveKeyword(
  keyword: string,
  buttons: readonly ImageAuthButton[],
): KeywordResolution {
  const normalised = normalise(keyword);
  const hits: number[] = [];
  for (let index = 0; index < buttons.length; index += 1) {
    const button = buttons[index];
    if (button === undefined) {
      continue;
    }
    if (normalise(button.altText) === normalised) {
      hits.push(index);
    }
  }
  if (hits.length === 0) {
    return { status: "missing_keyword", keyword };
  }
  if (hits.length > 1) {
    return {
      status: "ambiguous_keyword",
      keyword,
      indices: hits,
    };
  }
  const first = hits[0];
  if (first === undefined) {
    return { status: "missing_keyword", keyword };
  }
  return { status: "matched", index: first };
}

function normalise(value: string): string {
  return value.trim().replace(WHITESPACE_PATTERN, "");
}
