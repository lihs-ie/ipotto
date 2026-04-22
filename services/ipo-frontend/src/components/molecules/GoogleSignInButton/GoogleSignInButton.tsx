"use client";

import { useState } from "react";

import { Button } from "@/components/atoms/Button/Button";
import { useAuth } from "@/lib/firebase/auth-context";

import styles from "./GoogleSignInButton.module.css";

type Props = {
  label?: string;
  onError?: (error: Error) => void;
};

export const GoogleSignInButton = (props: Props) => {
  const auth = useAuth();
  const [pending, setPending] = useState(false);

  const handleClick = async (): Promise<void> => {
    setPending(true);
    try {
      await auth.signInWithGoogle();
    } catch (caught) {
      if (props.onError && caught instanceof Error) {
        props.onError(caught);
      }
    } finally {
      setPending(false);
    }
  };

  return (
    <div className={styles.container}>
      <Button
        label={props.label ?? "Googleでログイン"}
        disabled={pending || auth.state.status === "loading"}
        onClick={() => {
          void handleClick();
        }}
      />
    </div>
  );
};
