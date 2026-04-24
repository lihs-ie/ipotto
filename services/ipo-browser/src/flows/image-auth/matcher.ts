// Phase 3 Sprint 6 / Sprint 7 — pure logic that resolves Rakuten's
// image-auth keywords against the candidate list collected from the DOM.
//
// Input shape mirrors the domain ACL contract: two keywords emitted by
// the mail-driven OTP reader, each of which must match exactly one
// button. The authentication screen always asks for the keywords in
// order, so the returned indices preserve the order in which the
// operator must click the buttons.
//
// Phase 3 Sprint 7 extends the button shape with `onclickKeyword` so we
// can also match against Rakuten's production page where the alt
// attribute is absent and the keyword is encoded as the 4th argument to
// `emojiAltClick('id', index, 'src', 'charaWord')`.

export type ImageAuthenticationKeywords = {
  readonly first: string;
  readonly second: string;
};

export type ImageAuthButton = {
  readonly altText?: string | null;
  readonly onclickKeyword?: string | null;
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

// Matches `emojiAltClick('id', 0, '/src.gif', 'charaWord')` regardless of
// the quoting style. The 4th positional argument is the keyword.
const EMOJI_ALT_CLICK_PATTERN =
  /emojiAltClick\s*\(\s*['"][^'"]*['"]\s*,\s*[^,]+,\s*['"][^'"]*['"]\s*,\s*['"]([^'"]+)['"]\s*\)/;

/**
 * Resolves the two keywords to their button indices. The comparison is
 * exact match on the alt attribute (or `emojiAltClick` charaWord) after
 * trimming / collapsing whitespace so Rakuten's fullwidth vs halfwidth
 * space variations do not defeat the match. Each keyword is expected to
 * hit exactly one button; ambiguity (duplicate candidate) or absence is
 * surfaced so the caller can choose to fall back to the manual flow
 * without guessing.
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

/**
 * Extracts the charaWord (4th argument) from an `emojiAltClick(...)`
 * onclick attribute. Returns `null` when the attribute is absent or
 * uses the no-alt `emojiClick(...)` form (which cannot carry a
 * keyword).
 */
export function extractKeywordFromOnclick(
  onclickAttribute: string | null | undefined,
): string | null {
  if (onclickAttribute === null || onclickAttribute === undefined) {
    return null;
  }
  const match = EMOJI_ALT_CLICK_PATTERN.exec(onclickAttribute);
  if (match === null || match[1] === undefined) {
    return null;
  }
  return match[1];
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
    if (buttonMatches(button, normalised)) {
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

function buttonMatches(button: ImageAuthButton, normalisedKeyword: string): boolean {
  if (
    button.altText !== null &&
    button.altText !== undefined &&
    button.altText !== "" &&
    normalise(button.altText) === normalisedKeyword
  ) {
    return true;
  }
  if (
    button.onclickKeyword !== null &&
    button.onclickKeyword !== undefined &&
    button.onclickKeyword !== "" &&
    normalise(button.onclickKeyword) === normalisedKeyword
  ) {
    return true;
  }
  return false;
}

function normalise(value: string): string {
  return value.trim().replace(WHITESPACE_PATTERN, "");
}
