import { Firestore } from "@google-cloud/firestore";

import type {
  LotteryApplicationRecord,
} from "../../domain/notification-event.js";
import type { LotteryApplicationRepository } from "../../application/use-cases/apply-for-lottery-use-case.js";

/**
 * Firestore backed repository for lottery applications.
 */
export class FirestoreLotteryApplicationRepository
  implements LotteryApplicationRepository
{
  /**
   * Creates the repository.
   */
  public constructor(private readonly firestore: Firestore) {}

  /**
   * Returns whether a stock/account pair already exists.
   */
  public async existsByStockAndAccount(
    stockId: string,
    accountId: string,
  ): Promise<boolean> {
    const snapshot = await this.firestore
      .collection("lottery_applications")
      .where("stock", "==", stockId)
      .where("securitiesAccount", "==", accountId)
      .limit(1)
      .get();
    return !snapshot.empty;
  }

  /**
   * Saves a successful application record.
   */
  public async save(record: LotteryApplicationRecord): Promise<void> {
    await this.firestore
      .collection("lottery_applications")
      .doc(record.identifier)
      .set(record);
  }
}
