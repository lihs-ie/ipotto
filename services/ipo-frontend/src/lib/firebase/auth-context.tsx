"use client";

import {
  GoogleAuthProvider,
  onIdTokenChanged,
  signInWithPopup,
  signOut as firebaseSignOut,
  type Auth,
  type User,
} from "firebase/auth";
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";

import { type FirebaseConfig } from "./config";
import { getFirebaseClients } from "./client";

export type AuthStatus = "loading" | "unauthenticated" | "authenticated";

export type AuthState =
  | { status: "loading"; user: null }
  | { status: "unauthenticated"; user: null }
  | { status: "authenticated"; user: User };

export type AuthContextValue = {
  state: AuthState;
  getIdToken: () => Promise<string | null>;
  signInWithGoogle: () => Promise<void>;
  signOut: () => Promise<void>;
};

const AuthContext = createContext<AuthContextValue | null>(null);

type Props = {
  config: FirebaseConfig;
  children: ReactNode;
};

export const AuthProvider = (props: Props) => {
  const auth = useMemo<Auth>(
    () => getFirebaseClients(props.config).auth,
    [props.config],
  );
  const [state, setState] = useState<AuthState>({
    status: "loading",
    user: null,
  });

  useEffect(() => {
    const unsubscribe = onIdTokenChanged(auth, (user) => {
      if (user) {
        setState({ status: "authenticated", user });
      } else {
        setState({ status: "unauthenticated", user: null });
      }
    });
    return unsubscribe;
  }, [auth]);

  const getIdToken = useCallback(async (): Promise<string | null> => {
    const user = auth.currentUser;
    if (!user) return null;
    return user.getIdToken();
  }, [auth]);

  const signInWithGoogle = useCallback(async (): Promise<void> => {
    const provider = new GoogleAuthProvider();
    await signInWithPopup(auth, provider);
  }, [auth]);

  const signOut = useCallback(async (): Promise<void> => {
    await firebaseSignOut(auth);
  }, [auth]);

  const value = useMemo<AuthContextValue>(
    () => ({ state, getIdToken, signInWithGoogle, signOut }),
    [state, getIdToken, signInWithGoogle, signOut],
  );

  return <AuthContext.Provider value={value}>{props.children}</AuthContext.Provider>;
};

export const useAuth = (): AuthContextValue => {
  const value = useContext(AuthContext);
  if (!value) {
    throw new Error("useAuth must be used within AuthProvider");
  }
  return value;
};
