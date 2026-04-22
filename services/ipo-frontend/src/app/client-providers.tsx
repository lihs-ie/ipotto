"use client";

import { createContext, useContext, useMemo, type ReactNode } from "react";

import {
  AuthProvider,
  useAuth,
} from "@/lib/firebase/auth-context";
import { loadFirebaseConfig, type FirebaseConfig } from "@/lib/firebase/config";
import { createApiClient, createIpoApi, type IpoApi } from "@/lib/api";

const IpoApiContext = createContext<IpoApi | null>(null);

type ApiProviderProps = {
  baseUrl: string;
  children: ReactNode;
};

const ApiProvider = (props: ApiProviderProps) => {
  const auth = useAuth();
  const api = useMemo<IpoApi>(() => {
    const client = createApiClient({
      baseUrl: props.baseUrl,
      getIdToken: auth.getIdToken,
    });
    return createIpoApi(client);
  }, [auth.getIdToken, props.baseUrl]);
  return <IpoApiContext.Provider value={api}>{props.children}</IpoApiContext.Provider>;
};

export const useIpoApi = (): IpoApi => {
  const value = useContext(IpoApiContext);
  if (!value) {
    throw new Error("useIpoApi must be used within ClientProviders");
  }
  return value;
};

type Props = {
  config: FirebaseConfig;
  apiBaseUrl: string;
  children: ReactNode;
};

const ClientProvidersImpl = (props: Props) => (
  <AuthProvider config={props.config}>
    <ApiProvider baseUrl={props.apiBaseUrl}>{props.children}</ApiProvider>
  </AuthProvider>
);

type BootstrapProps = {
  children: ReactNode;
};

/// Convenience wrapper that reads `NEXT_PUBLIC_*` env vars and hands
/// the config to the nested providers. Kept as its own component so
/// `app/layout.tsx` can stay a server component and delegate the
/// client boundary to this file.
export const ClientProviders = (props: BootstrapProps) => {
  const config = loadFirebaseConfig();
  const apiBaseUrl =
    process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8080";
  return (
    <ClientProvidersImpl config={config} apiBaseUrl={apiBaseUrl}>
      {props.children}
    </ClientProvidersImpl>
  );
};
