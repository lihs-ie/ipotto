"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";

import { useAuth } from "@/lib/firebase/auth-context";

import styles from "./LoginClient.module.css";

export const LoginClient = () => {
  const auth = useAuth();
  const router = useRouter();
  const [pending, setPending] = useState(false);

  useEffect(() => {
    if (auth.state.status === "authenticated") {
      router.replace("/");
    }
  }, [auth.state.status, router]);

  const handleSignIn = async (): Promise<void> => {
    setPending(true);
    try {
      await auth.signInWithGoogle();
    } finally {
      setPending(false);
    }
  };

  return (
    <main className={styles.container}>
      <div className={styles.decoration} />

      <div className={styles.content}>
        <div className={styles.branding}>
          <div className={styles.logo}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
              <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
            </svg>
          </div>
          <span className={styles.brand}>IPOtto</span>
          <span className={styles.tagline}>IPO Lottery Automation</span>
        </div>

        <div className={styles.card}>
          <h1 className={styles.heading}>ようこそ</h1>
          <p className={styles.description}>
            IPO抽選申し込みを自動化し、進捗を一元管理するツール
          </p>

          <button
            className={styles.button}
            data-variant="ghost"
            type="button"
            disabled={pending || auth.state.status === "loading"}
            onClick={() => {
              void handleSignIn();
            }}
          >
            <svg viewBox="0 0 24 24">
              <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 0 1-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z" fill="#4285F4" />
              <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853" />
              <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18A10.96 10.96 0 0 0 1 12c0 1.77.42 3.45 1.18 4.93l3.66-2.84z" fill="#FBBC05" />
              <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335" />
            </svg>
            Googleでログイン
          </button>

          <div className={styles.notice}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
            </svg>
            <span>
              個人利用専用ツールです。認証情報はGCP Secret Managerで暗号化して保管されます。
            </span>
          </div>
        </div>

        <p className={styles.version}>v1.0.0 · MVP</p>
      </div>
    </main>
  );
};
