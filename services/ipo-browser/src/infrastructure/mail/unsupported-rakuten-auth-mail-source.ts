import type { AccountCredential } from "../../domain/account-credential.js";

import type { RakutenAuthMailSource } from "./rakuten-auth-mail-parser.js";

/**
 * Placeholder mail source used until a concrete IMAP or Gmail reader is wired in.
 */
export class UnsupportedRakutenAuthMailSource
  implements RakutenAuthMailSource
{
  /**
   * Always fails because no real mail transport is configured yet.
   */
  public async fetchLatestAuthenticationMail(
    credential: AccountCredential,
    receivedAfter: Date,
  ): Promise<string | null> {
    void credential;
    void receivedAfter;
    throw new Error(
      "Rakuten authentication mail source is not configured",
    );
  }
}
