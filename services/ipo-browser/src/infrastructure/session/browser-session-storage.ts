import { mkdir, readdir, rm, stat } from "node:fs/promises";
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

  /**
   * Removes session directories older than the configured retention period.
   */
  public async cleanupExpiredSessions(
    retentionHours: number,
    nowProvider: { readonly now: () => Date } = { now: () => new Date() },
  ): Promise<number> {
    await mkdir(this.baseDirectory, { recursive: true });
    const entries = await readdir(this.baseDirectory, { withFileTypes: true });
    const cutoffTime =
      nowProvider.now().getTime() - retentionHours * 60 * 60 * 1000;
    let deletedCount = 0;

    for (const entry of entries) {
      if (!entry.isDirectory()) {
        continue;
      }

      const directoryPath = path.join(this.baseDirectory, entry.name);
      const details = await stat(directoryPath);
      if (details.mtime.getTime() >= cutoffTime) {
        continue;
      }

      await rm(directoryPath, { recursive: true, force: true });
      deletedCount += 1;
    }

    return deletedCount;
  }
}
