import { mkdir } from "node:fs/promises";
import path from "node:path";

/**
 * Browser session storage for Playwright persistent contexts.
 */
export class BrowserSessionStorage {
  /**
   * Creates the storage.
   */
  public constructor(private readonly baseDirectory: string) {}

  /**
   * Returns the persistent context directory for an account.
   */
  public getUserDataDirectory(accountId: string): string {
    return path.join(this.baseDirectory, accountId);
  }

  /**
   * Ensures the directory exists and returns it.
   */
  public async ensureUserDataDirectory(accountId: string): Promise<string> {
    const directory = this.getUserDataDirectory(accountId);
    await mkdir(directory, { recursive: true });
    return directory;
  }
}
