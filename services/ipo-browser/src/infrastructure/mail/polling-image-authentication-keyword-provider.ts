import { setTimeout as sleep } from "node:timers/promises";

import type { AccountCredential } from "../../domain/account-credential.js";
import type {
  ImageAuthenticationKeyword,
  ImageAuthenticationKeywordProvider,
} from "../rakuten/rakuten-broker-adapter.js";
import {
  extractImageAuthenticationKeywords,
  MailRetrievalTimeoutError,
  type RakutenAuthMailSource,
} from "./rakuten-auth-mail-parser.js";

/**
 * Runtime configuration for mail polling.
 */
export interface MailPollingConfig {
  readonly pollingIntervalMs: number;
  readonly timeoutMs: number;
}

/**
 * Provider that polls the mail source until Rakuten authentication keywords arrive.
 */
export class PollingImageAuthenticationKeywordProvider
  implements ImageAuthenticationKeywordProvider
{
  /**
   * Creates the provider.
   */
  public constructor(
    private readonly source: RakutenAuthMailSource,
    private readonly config: MailPollingConfig,
    private readonly nowProvider: { readonly now: () => Date } = {
      now: () => new Date(),
    },
    private readonly sleepFunction: (milliseconds: number) => Promise<void> = (
      milliseconds,
    ) => sleep(milliseconds).then(() => undefined),
  ) {}

  /**
   * Fetches image authentication keywords from Rakuten mails.
   */
  public async fetchKeywords(
    credential: AccountCredential,
  ): Promise<ImageAuthenticationKeyword> {
    const startedAt = this.nowProvider.now();

    while (this.nowProvider.now().getTime() - startedAt.getTime() < this.config.timeoutMs) {
      const mailBody = await this.source.fetchLatestAuthenticationMail(
        credential,
        startedAt,
      );

      if (mailBody !== null) {
        return extractImageAuthenticationKeywords(mailBody);
      }

      await this.sleepFunction(this.config.pollingIntervalMs);
    }

    throw new MailRetrievalTimeoutError(
      `authentication mail was not received within ${this.config.timeoutMs}ms`,
    );
  }
}
