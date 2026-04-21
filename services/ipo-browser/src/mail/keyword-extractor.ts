import type { ImageAuthenticationKeyword } from "./types.js";

// Phase 3 Sprint 6.2 — Rakuten's 2FA mail contains the image-auth
// keyword pair as "<keyword1> + <keyword2>" (full-width or
// half-width plus sign, arbitrary whitespace). The Rust-side
// `image_authentication_keyword_parser` uses the same expression so
// we stay bit-for-bit compatible: mail body fed through either
// implementation yields the same (first, second) pair.

const KEYWORD_PATTERN = /([^\s+]+)\s*\+\s*([^\s+]+)/u;

/**
 * Extracts the ordered image-auth keyword pair from a Rakuten 2FA
 * mail body. Returns `null` when the pattern is not present (caller
 * should keep polling for a fresher message).
 */
export function parseImageAuthenticationKeyword(
  body: string,
): ImageAuthenticationKeyword | null {
  const match = body.match(KEYWORD_PATTERN);
  if (match === null) {
    return null;
  }
  const first = match[1];
  const second = match[2];
  if (first === undefined || second === undefined) {
    return null;
  }
  if (first.length === 0 || second.length === 0) {
    return null;
  }
  return { first, second };
}
