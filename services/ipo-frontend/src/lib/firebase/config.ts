import { z } from "zod";

const rawConfigSchema = z.object({
  NEXT_PUBLIC_FIREBASE_API_KEY: z.string().optional(),
  NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN: z.string().optional(),
  NEXT_PUBLIC_FIREBASE_PROJECT_ID: z.string().optional(),
  NEXT_PUBLIC_FIREBASE_AUTH_EMULATOR_HOST: z.string().optional(),
});

export type FirebaseConfig = {
  apiKey: string;
  authDomain: string;
  projectId: string;
  authEmulatorHost: string | null;
};

const DEFAULT_CONFIG: FirebaseConfig = {
  apiKey: "fake-api-key",
  authDomain: "ipotto-local.firebaseapp.com",
  projectId: "ipotto-local",
  authEmulatorHost: null,
};

/// Reads the `NEXT_PUBLIC_FIREBASE_*` env vars. Missing values fall
/// back to local-emulator defaults (`ipotto-local`) so `next build`
/// can statically render pages without requiring real Firebase
/// credentials at compile time. Production deployments pass real
/// values via the container environment, which Next.js inlines into
/// the client bundle at build time.
export const loadFirebaseConfig = (
  env: Record<string, string | undefined> = {
    NEXT_PUBLIC_FIREBASE_API_KEY: process.env.NEXT_PUBLIC_FIREBASE_API_KEY,
    NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN:
      process.env.NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN,
    NEXT_PUBLIC_FIREBASE_PROJECT_ID:
      process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID,
    NEXT_PUBLIC_FIREBASE_AUTH_EMULATOR_HOST:
      process.env.NEXT_PUBLIC_FIREBASE_AUTH_EMULATOR_HOST,
  },
): FirebaseConfig => {
  const parsed = rawConfigSchema.parse(env);
  const emulatorHost =
    parsed.NEXT_PUBLIC_FIREBASE_AUTH_EMULATOR_HOST !== undefined &&
    parsed.NEXT_PUBLIC_FIREBASE_AUTH_EMULATOR_HOST.trim() !== ""
      ? parsed.NEXT_PUBLIC_FIREBASE_AUTH_EMULATOR_HOST
      : null;
  return {
    apiKey: parsed.NEXT_PUBLIC_FIREBASE_API_KEY ?? DEFAULT_CONFIG.apiKey,
    authDomain:
      parsed.NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN ?? DEFAULT_CONFIG.authDomain,
    projectId:
      parsed.NEXT_PUBLIC_FIREBASE_PROJECT_ID ?? DEFAULT_CONFIG.projectId,
    authEmulatorHost: emulatorHost,
  };
};
