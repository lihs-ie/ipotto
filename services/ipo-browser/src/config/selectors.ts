import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

// Phase 3 Sprint 5.3 — selectors are loaded once at startup from the YAML
// file sitting next to this module. Operators can swap the YAML without
// rebuilding the image when Rakuten changes their DOM.

export type SelectorDefinition = {
  primary: string;
  fallback?: readonly string[];
};

export type RakutenLoginSelectors = {
  loginIdInput: SelectorDefinition;
  passwordInput: SelectorDefinition;
  submitButton: SelectorDefinition;
};

export type RakutenImageAuthenticationSelectors = {
  container: SelectorDefinition;
  imageButtons: SelectorDefinition;
  imageElement: SelectorDefinition;
  submitButton: SelectorDefinition;
  successIndicator: SelectorDefinition;
  errorMessage: SelectorDefinition;
};

export type RakutenApplyListSelectors = {
  stockRow: SelectorDefinition;
  applyButton: SelectorDefinition;
};

export type RakutenApplyFormSelectors = {
  sharesInput: SelectorDefinition;
  priceInput: SelectorDefinition;
  tradingPasswordInput: SelectorDefinition;
  submitButton: SelectorDefinition;
};

export type RakutenApplyResultSelectors = {
  successIndicator: SelectorDefinition;
  duplicateIndicator: SelectorDefinition;
  insufficientBalanceIndicator: SelectorDefinition;
  failureIndicator: SelectorDefinition;
};

export type RakutenLotteryResultSelectors = {
  resultContainer: SelectorDefinition;
  resultRow: SelectorDefinition;
  resultLabel: SelectorDefinition;
};

export type SelectorTree = {
  rakuten: {
    login: RakutenLoginSelectors;
    imageAuthentication: RakutenImageAuthenticationSelectors;
    applyList: RakutenApplyListSelectors;
    applyForm: RakutenApplyFormSelectors;
    applyResult: RakutenApplyResultSelectors;
    lotteryResult: RakutenLotteryResultSelectors;
  };
};

let cached: SelectorTree | null = null;

export function loadSelectors(): SelectorTree {
  if (cached !== null) {
    return cached;
  }
  const dir = dirname(fileURLToPath(import.meta.url));
  const yamlPath =
    process.env["IPO_BROWSER_SELECTORS_PATH"] ?? join(dir, "selectors.yaml");
  const raw = readFileSync(yamlPath, "utf-8");
  const parsed = yaml.load(raw) as SelectorTree;
  cached = parsed;
  return parsed;
}

export function allSelectors(definition: SelectorDefinition): readonly string[] {
  return [definition.primary, ...(definition.fallback ?? [])];
}
