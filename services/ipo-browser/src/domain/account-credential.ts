/**
 * Mail credential used to retrieve OTP or image authentication hints.
 */
export interface MailCredential {
  readonly mailAddress: string;
  readonly mailPassword: string;
  readonly imapHost: string;
  readonly imapPort: number;
}

/**
 * Broker account credential required for browser automation.
 */
export interface AccountCredential {
  readonly loginId: string;
  readonly loginPassword: string;
  readonly tradingPassword: string;
  readonly mailCredential: MailCredential;
}

/**
 * Active securities account loaded from Firestore and Secret Manager.
 */
export interface ActiveSecuritiesAccount {
  readonly identifier: string;
  readonly securitiesCompany: string;
  readonly credential: AccountCredential;
}

/**
 * Connection test result returned by the browser adapter.
 */
export interface ConnectionTestResult {
  readonly success: boolean;
  readonly message: string;
  readonly testedAt: string;
}
