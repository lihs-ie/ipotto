import { Firestore } from "@google-cloud/firestore";

import type { ExclusionEntry } from "../../domain/ipo-stock.js";
import type { ExclusionRepository } from "../../application/use-cases/apply-for-lottery-use-case.js";

interface ExclusionDocument {
  readonly identifier: string;
  readonly companyName: string;
  readonly reason: string;
}

/**
 * Firestore backed repository for exclusions.
 */
export class FirestoreExclusionRepository implements ExclusionRepository {
  /**
   * Creates the repository.
   */
  public constructor(private readonly firestore: Firestore) {}

  /**
   * Returns all exclusions.
   */
  public async findAll(): Promise<readonly ExclusionEntry[]> {
    const snapshot = await this.firestore.collection("exclusions").get();
    return snapshot.docs.map((document) => {
      const data = document.data() as ExclusionDocument;
      return {
        identifier: data.identifier,
        companyName: data.companyName,
        reason: data.reason,
      } satisfies ExclusionEntry;
    });
  }
}
