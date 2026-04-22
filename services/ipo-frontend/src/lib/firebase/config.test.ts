import { describe, expect, it } from "vitest";

import { loadFirebaseConfig } from "./config";

describe("loadFirebaseConfig", () => {
  it("returns a complete config when all required vars are set", () => {
    const config = loadFirebaseConfig({
      NEXT_PUBLIC_FIREBASE_API_KEY: "api-key",
      NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN: "ipotto-stg.firebaseapp.com",
      NEXT_PUBLIC_FIREBASE_PROJECT_ID: "ipotto-stg",
      NEXT_PUBLIC_FIREBASE_AUTH_EMULATOR_HOST: undefined,
    });
    expect(config).toEqual({
      apiKey: "api-key",
      authDomain: "ipotto-stg.firebaseapp.com",
      projectId: "ipotto-stg",
      authEmulatorHost: null,
    });
  });

  it("activates emulator mode when emulator host is provided", () => {
    const config = loadFirebaseConfig({
      NEXT_PUBLIC_FIREBASE_API_KEY: "fake",
      NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN: "ipotto-local.firebaseapp.com",
      NEXT_PUBLIC_FIREBASE_PROJECT_ID: "ipotto-local",
      NEXT_PUBLIC_FIREBASE_AUTH_EMULATOR_HOST: "firebase-emulator:9099",
    });
    expect(config.authEmulatorHost).toBe("firebase-emulator:9099");
  });

  it("treats empty emulator host string as absent", () => {
    const config = loadFirebaseConfig({
      NEXT_PUBLIC_FIREBASE_API_KEY: "fake",
      NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN: "ipotto-local.firebaseapp.com",
      NEXT_PUBLIC_FIREBASE_PROJECT_ID: "ipotto-local",
      NEXT_PUBLIC_FIREBASE_AUTH_EMULATOR_HOST: "",
    });
    expect(config.authEmulatorHost).toBeNull();
  });

  it("falls back to local-emulator defaults when env vars are missing", () => {
    const config = loadFirebaseConfig({});
    expect(config.projectId).toBe("ipotto-local");
    expect(config.apiKey).toBe("fake-api-key");
    expect(config.authDomain).toBe("ipotto-local.firebaseapp.com");
    expect(config.authEmulatorHost).toBeNull();
  });
});
