import { promises as fs } from "node:fs";
import { join } from "node:path";

import type { Page } from "playwright";

import {
  allSelectors,
  loadSelectors,
  type SelectorDefinition,
} from "../../config/selectors.js";
import { logger } from "../../logger.js";

import {
  chooseImageIndices,
  extractKeywordFromOnclick,
  type ImageAuthButton,
  type ImageAuthenticationKeywords,
} from "./matcher.js";

// Phase 3 Sprint 6 — Playwright image-authentication flow. Hidden
// behind a single entry point so the login flow (PR #c) can call it
// after it detects that Rakuten has served the 2FA screen. The flow
// is deliberately single-shot: the ACL contract forbids retries
// because 3 consecutive image-auth failures lock the broker account.
// Callers that want a second attempt must open a fresh login session.

export type ImageAuthOptions = {
  readonly keywords: ImageAuthenticationKeywords;
};

export type ImageAuthResult =
  | { readonly status: "success" }
  | {
      readonly status: "failure";
      readonly reason: string;
      readonly screenshotPath: string | null;
    };

const SCREENSHOT_DIR =
  process.env["BROWSER_SCREENSHOT_DIR"] ?? "/tmp/ipo-browser-screenshots";
const CONTAINER_WAIT_MS = 15_000;
const SUBMIT_WAIT_MS = 10_000;

export async function runImageAuthentication(
  page: Page,
  options: ImageAuthOptions,
): Promise<ImageAuthResult> {
  const selectors = loadSelectors().rakuten.imageAuthentication;

  try {
    const containerSelector = await waitForFirstVisible(
      page,
      selectors.container,
      CONTAINER_WAIT_MS,
    );
    if (containerSelector === null) {
      return failure(
        page,
        "container_not_found",
        `image auth container not visible within ${CONTAINER_WAIT_MS}ms (tried ${allSelectors(selectors.container).join(", ")})`,
      );
    }

    const buttonsSelector = await firstMatchingSelector(
      page,
      selectors.imageButtons,
    );
    if (buttonsSelector === null) {
      return failure(
        page,
        "buttons_not_found",
        `no image buttons matched (tried ${allSelectors(selectors.imageButtons).join(", ")})`,
      );
    }

    const buttonHandles = await page.locator(buttonsSelector).all();
    if (buttonHandles.length === 0) {
      return failure(
        page,
        "buttons_not_found",
        "image buttons locator resolved to zero elements",
      );
    }

    const candidateButtons: ImageAuthButton[] = await Promise.all(
      buttonHandles.map(async (handle) => {
        const direct = await handle.getAttribute("alt").catch(() => null);
        const nestedAlt =
          direct !== null && direct.length > 0
            ? direct
            : await handle
                .locator(selectors.imageElement.primary)
                .first()
                .getAttribute("alt")
                .catch(() => null);
        const onclickAttribute = await handle
          .getAttribute("onclick")
          .catch(() => null);
        return {
          altText: nestedAlt ?? null,
          onclickKeyword: extractKeywordFromOnclick(onclickAttribute),
        };
      }),
    );

    const outcome = chooseImageIndices(options.keywords, candidateButtons);
    if (outcome.status !== "matched") {
      return failure(
        page,
        `matcher_${outcome.status}`,
        JSON.stringify(outcome),
      );
    }

    for (const index of outcome.indices) {
      const target = buttonHandles[index];
      if (target === undefined) {
        return failure(
          page,
          "index_out_of_bounds",
          `matcher returned index ${index} but only ${buttonHandles.length} buttons were collected`,
        );
      }
      await target.click();
    }

    const submitSelector = await firstMatchingSelector(
      page,
      selectors.submitButton,
    );
    if (submitSelector === null) {
      return failure(
        page,
        "submit_not_found",
        `no submit button matched (tried ${allSelectors(selectors.submitButton).join(", ")})`,
      );
    }
    await page.click(submitSelector);

    await page
      .waitForLoadState("networkidle", { timeout: SUBMIT_WAIT_MS })
      .catch(() => undefined);

    const errorSelector = await firstVisibleSelector(page, selectors.errorMessage);
    if (errorSelector !== null) {
      const message = await page.locator(errorSelector).first().innerText().catch(() => "");
      return failure(page, "error_indicator_visible", message || "error indicator visible");
    }
    const successSelector = await firstVisibleSelector(
      page,
      selectors.successIndicator,
    );
    if (successSelector !== null) {
      return { status: "success" };
    }
    // Neither indicator visible yet: treat as success if no error is
    // surfaced after networkidle — callers can re-check later.
    return { status: "success" };
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error);
    logger.error({
      event: "flows.image_auth.exception",
      error: error instanceof Error ? error.message : String(error),
    });
    const screenshotPath = await tryScreenshot(page, "image-auth-exception");
    return { status: "failure", reason, screenshotPath };
  }
}

async function waitForFirstVisible(
  page: Page,
  definition: SelectorDefinition,
  timeoutMs: number,
): Promise<string | null> {
  for (const selector of allSelectors(definition)) {
    const locator = page.locator(selector).first();
    const visible = await locator
      .waitFor({ state: "visible", timeout: timeoutMs })
      .then(() => true)
      .catch(() => false);
    if (visible) {
      return selector;
    }
  }
  return null;
}

async function firstMatchingSelector(
  page: Page,
  definition: SelectorDefinition,
): Promise<string | null> {
  for (const selector of allSelectors(definition)) {
    const count = await page
      .locator(selector)
      .count()
      .catch(() => 0);
    if (count > 0) {
      return selector;
    }
  }
  return null;
}

async function firstVisibleSelector(
  page: Page,
  definition: SelectorDefinition,
): Promise<string | null> {
  for (const selector of allSelectors(definition)) {
    const visible = await page
      .locator(selector)
      .first()
      .isVisible()
      .catch(() => false);
    if (visible) {
      return selector;
    }
  }
  return null;
}

async function failure(
  page: Page,
  code: string,
  message: string,
): Promise<ImageAuthResult> {
  const screenshotPath = await tryScreenshot(page, `image-auth-${code}`);
  return {
    status: "failure",
    reason: `${code}: ${message}`,
    screenshotPath,
  };
}

async function tryScreenshot(
  page: Page,
  label: string,
): Promise<string | null> {
  try {
    await fs.mkdir(SCREENSHOT_DIR, { recursive: true });
    const path = join(SCREENSHOT_DIR, `${label}-${Date.now()}.png`);
    await page.screenshot({ path, fullPage: true });
    return path;
  } catch {
    return null;
  }
}
