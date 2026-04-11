import { Firestore } from "@google-cloud/firestore";

import type { OperationLogEntry } from "../../domain/notification-event.js";
import type { OperationLogRepository } from "../../application/use-cases/apply-for-lottery-use-case.js";

/**
 * Firestore backed repository for operation logs.
 */
export class FirestoreOperationLogRepository implements OperationLogRepository {
  /**
   * Creates the repository.
   */
  public constructor(private readonly firestore: Firestore) {}

  /**
   * Saves an operation log entry.
   */
  public async save(entry: OperationLogEntry): Promise<void> {
    await this.firestore.collection("operation_logs").doc(entry.identifier).set(entry);
  }
}
