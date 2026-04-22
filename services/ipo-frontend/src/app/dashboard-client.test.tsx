import { render, screen } from "@testing-library/react";
import type { User } from "firebase/auth";
import { describe, expect, it, vi } from "vitest";

import { DashboardClient } from "./dashboard-client";
import type { AuthContextValue, AuthState } from "@/lib/firebase/auth-context";

const replace = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({ replace, push: vi.fn(), back: vi.fn() }),
}));

type MockAuthContext = AuthContextValue;

let mockValue: MockAuthContext = {
  state: { status: "loading", user: null } as AuthState,
  getIdToken: async () => null,
  signInWithGoogle: async () => undefined,
  signOut: async () => undefined,
};

vi.mock("@/lib/firebase/auth-context", async () => {
  const actual =
    await vi.importActual<typeof import("@/lib/firebase/auth-context")>(
      "@/lib/firebase/auth-context",
    );
  return {
    ...actual,
    useAuth: () => mockValue,
  };
});

const setAuth = (state: AuthState): void => {
  mockValue = { ...mockValue, state };
};

describe("DashboardClient", () => {
  it("shows loading copy while auth state is resolving", () => {
    replace.mockClear();
    setAuth({ status: "loading", user: null });
    render(<DashboardClient />);
    expect(screen.getByText("認証状態を確認しています…")).toBeDefined();
    expect(replace).not.toHaveBeenCalled();
  });

  it("redirects to /login when unauthenticated", () => {
    replace.mockClear();
    setAuth({ status: "unauthenticated", user: null });
    render(<DashboardClient />);
    expect(replace).toHaveBeenCalledWith("/login");
  });

  it("renders the dashboard heading when authenticated", () => {
    replace.mockClear();
    setAuth({
      status: "authenticated",
      user: { email: "user@example.com" } as unknown as User,
    });
    render(<DashboardClient />);
    expect(
      screen.getByRole("heading", { level: 1 }).textContent,
    ).toContain("IPOtto Dashboard");
  });
});
