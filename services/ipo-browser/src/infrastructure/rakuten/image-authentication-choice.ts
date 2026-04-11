/**
 * Structured image-authentication choice extracted from the page markup.
 */
export interface ImageAuthenticationChoice {
  readonly buttonId: string | null;
  readonly buttonIndex: number;
  readonly labels: readonly string[];
}

/**
 * Serialized button attributes used to derive a choice.
 */
export interface ImageAuthenticationChoiceAttributes {
  readonly buttonId: string | null;
  readonly buttonIndex: number;
  readonly onclick: string | null;
  readonly imageAlt: string | null;
  readonly imageSrc: string | null;
}

/**
 * Builds a normalized choice from button attributes.
 */
export function buildImageAuthenticationChoice(
  attributes: ImageAuthenticationChoiceAttributes,
): ImageAuthenticationChoice {
  const labels = collectLabels(attributes);
  return {
    buttonId: attributes.buttonId,
    buttonIndex: attributes.buttonIndex,
    labels,
  };
}

/**
 * Finds a choice matching the given keyword.
 */
export function findMatchingImageAuthenticationChoice(
  choices: readonly ImageAuthenticationChoice[],
  keyword: string,
): ImageAuthenticationChoice | null {
  const normalizedKeyword = normalizeImageAuthenticationLabel(keyword);
  for (const choice of choices) {
    if (
      choice.labels.some(
        (label) =>
          normalizeImageAuthenticationLabel(label) === normalizedKeyword,
      )
    ) {
      return choice;
    }
  }
  return null;
}

/**
 * Normalizes a user-facing image-authentication label.
 */
export function normalizeImageAuthenticationLabel(label: string): string {
  return label.trim().toLowerCase();
}

/**
 * Extracts a label embedded in an emojiAltClick handler.
 */
export function extractAltLabelFromOnclick(
  onclick: string | null,
): string | null {
  if (onclick === null) {
    return null;
  }

  const match = onclick.match(
    /emojiAltClick\(\s*'[^']*'\s*,\s*'[^']*'\s*,\s*'[^']*'\s*,\s*'([^']+)'\s*\)/,
  );
  const value = match?.[1];
  return value === undefined ? null : value;
}

/**
 * Extracts the emoji asset identifier from the image source.
 */
export function extractImageCodeFromSource(
  imageSrc: string | null,
): string | null {
  if (imageSrc === null) {
    return null;
  }

  const match = imageSrc.match(/\/([^/]+)\.[a-z0-9]+(?:\?|$)/i);
  const value = match?.[1];
  return value === undefined ? null : value;
}

/**
 * Collects all matching labels for a choice in priority order.
 */
function collectLabels(
  attributes: ImageAuthenticationChoiceAttributes,
): readonly string[] {
  const candidates = [
    attributes.imageAlt,
    extractAltLabelFromOnclick(attributes.onclick),
    extractImageCodeFromSource(attributes.imageSrc),
  ];

  const labels: string[] = [];
  for (const candidate of candidates) {
    if (candidate === null) {
      continue;
    }
    const trimmed = candidate.trim();
    if (trimmed === "") {
      continue;
    }
    if (
      labels.some(
        (label) =>
          normalizeImageAuthenticationLabel(label) ===
          normalizeImageAuthenticationLabel(trimmed),
      )
    ) {
      continue;
    }
    labels.push(trimmed);
  }
  return labels;
}
