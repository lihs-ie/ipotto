import { test as base, expect } from "@playwright/test";

const EMULATOR_HOST =
  process.env.FIREBASE_AUTH_EMULATOR_HOST ?? "localhost:9099";
const API_BASE_URL = process.env.API_BASE_URL ?? "http://localhost:8080";
const API_KEY = "fake-api-key";
const TEST_EMAIL = "e2e-test@example.com";
const TEST_PASSWORD = "e2e-test-password-123";

async function mintIdToken(): Promise<string> {
  const signUpUrl = `http://${EMULATOR_HOST}/identitytoolkit.googleapis.com/v1/accounts:signUp?key=${API_KEY}`;
  const signUpResponse = await fetch(signUpUrl, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      email: TEST_EMAIL,
      password: TEST_PASSWORD,
      returnSecureToken: true,
    }),
  });

  if (signUpResponse.ok) {
    const data = await signUpResponse.json();
    return data.idToken as string;
  }

  const signInUrl = `http://${EMULATOR_HOST}/identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key=${API_KEY}`;
  const signInResponse = await fetch(signInUrl, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      email: TEST_EMAIL,
      password: TEST_PASSWORD,
      returnSecureToken: true,
    }),
  });

  if (!signInResponse.ok) {
    throw new Error(
      `Failed to sign in test user: ${await signInResponse.text()}`,
    );
  }

  const data = await signInResponse.json();
  return data.idToken as string;
}

type ApiFixtures = {
  idToken: string;
  apiBaseUrl: string;
};

export const test = base.extend<ApiFixtures>({
  idToken: async ({}, use) => {
    const token = await mintIdToken();
    await use(token);
  },
  apiBaseUrl: async ({}, use) => {
    await use(API_BASE_URL);
  },
});

export { expect };
export { TEST_EMAIL };
