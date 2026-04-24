import { type FirebaseApp, getApps, initializeApp } from "firebase/app";
import { type Auth, connectAuthEmulator, getAuth } from "firebase/auth";

import { type FirebaseConfig } from "./config";

type FirebaseClients = {
  app: FirebaseApp;
  auth: Auth;
};

let cached: FirebaseClients | null = null;

/// Initializes (or retrieves) the Firebase app + Auth client. When the
/// config has an emulator host, `connectAuthEmulator` re-routes all auth
/// calls to the local Firebase emulator.
export const getFirebaseClients = (config: FirebaseConfig): FirebaseClients => {
  if (cached) return cached;

  const existing = getApps()[0];
  const app =
    existing ??
    initializeApp({
      apiKey: config.apiKey,
      authDomain: config.authDomain,
      projectId: config.projectId,
    });
  const auth = getAuth(app);
  if (config.authEmulatorHost) {
    connectAuthEmulator(auth, `http://${config.authEmulatorHost}`, {
      disableWarnings: true,
    });
  }
  cached = { app, auth };
  return cached;
};

/// Resets the cached clients. Intended for tests only.
export const resetFirebaseClientCache = (): void => {
  cached = null;
};
