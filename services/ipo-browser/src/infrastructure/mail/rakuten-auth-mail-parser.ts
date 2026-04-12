import type { AccountCredential } from "../../domain/account-credential.js";
import type {
  ImageAuthenticationKeyword,
} from "../rakuten/rakuten-broker-adapter.js";

/**
 * Error raised when the Rakuten authentication mail cannot be parsed.
 */
export class RakutenAuthMailParseError extends Error {
  /**
   * Creates the parser error.
   */
  public constructor(message: string) {
    super(message);
    this.name = "RakutenAuthMailParseError";
  }
}

/**
 * Error raised when the authentication mail cannot be retrieved in time.
 */
export class MailRetrievalTimeoutError extends Error {
  /**
   * Creates the timeout error.
   */
  public constructor(message: string) {
    super(message);
    this.name = "MailRetrievalTimeoutError";
  }
}

/**
 * Error raised when the Rakuten authentication mail source cannot be queried.
 */
export class RakutenAuthMailSourceError extends Error {
  /**
   * Creates the mail source error.
   */
  public constructor(message: string) {
    super(message);
    this.name = "RakutenAuthMailSourceError";
  }
}

/**
 * Mail source used to retrieve Rakuten authentication mails.
 */
export interface RakutenAuthMailSource {
  /**
   * Returns the latest Rakuten authentication mail body when available.
   */
  fetchLatestAuthenticationMail(
    credential: AccountCredential,
    receivedAfter: Date,
  ): Promise<string | null>;
}

/**
 * Extracts image authentication keywords from a Rakuten mail body.
 */
export function extractImageAuthenticationKeywords(
  mailBody: string,
): ImageAuthenticationKeyword {
  const keywordLine = mailBody
    .split(/\r?\n/u)
    .map((line) => line.trim())
    .find((line) => line.includes("+"));

  if (keywordLine === undefined) {
    throw new RakutenAuthMailParseError(
      "authentication keyword line was not found in the mail body",
    );
  }

  const keywords = keywordLine
    .split("+")
    .map((keyword) => keyword.trim())
    .filter((keyword) => keyword !== "");

  if (keywords.length !== 2) {
    throw new RakutenAuthMailParseError(
      `expected exactly two authentication keywords but found ${keywords.length}`,
    );
  }

  const [firstKeyword, secondKeyword] = keywords;
  if (firstKeyword === undefined || secondKeyword === undefined) {
    throw new RakutenAuthMailParseError(
      "authentication keywords are incomplete",
    );
  }

  return { firstKeyword, secondKeyword };
}
