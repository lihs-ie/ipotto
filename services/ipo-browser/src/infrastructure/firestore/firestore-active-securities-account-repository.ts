import { Firestore } from "@google-cloud/firestore";

import type { ActiveSecuritiesAccount } from "../../domain/account-credential.js";
import type { ActiveSecuritiesAccountRepository } from "../../application/use-cases/apply-for-lottery-use-case.js";
import { SecretManagerAccountCredentialStore } from "../secrets/secret-manager-account-credential-store.js";

interface SecuritiesAccountDocument {
  readonly identifier: string;
  readonly securitiesCompany: string;
  readonly credentialSecretKey: string;
  readonly activation?: {
    readonly isActive?: boolean;
  };
}

/**
 * Firestore backed repository for active securities accounts.
 */
export class FirestoreActiveSecuritiesAccountRepository
  implements ActiveSecuritiesAccountRepository
{
  /**
   * Creates the repository.
   */
  public constructor(
    private readonly firestore: Firestore,
    private readonly credentialStore: SecretManagerAccountCredentialStore,
  ) {}

  /**
   * Returns active accounts with credentials restored from Secret Manager.
   */
  public async findActive(): Promise<readonly ActiveSecuritiesAccount[]> {
    const snapshot = await this.firestore
      .collection("securities_accounts")
      .where("activation.isActive", "==", true)
      .get();

    const accounts = await Promise.all(
      snapshot.docs.map(async (document) => {
        const data = document.data() as SecuritiesAccountDocument;
        const credential = await this.credentialStore.getAccountCredential(
          data.credentialSecretKey,
        );
        return {
          identifier: data.identifier,
          securitiesCompany: data.securitiesCompany,
          credential,
        } satisfies ActiveSecuritiesAccount;
      }),
    );

    return accounts;
  }
}
