"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";

import { Container } from "@/components/atoms/Container/Container";
import { Typography } from "@/components/atoms/Typography/Typography";
import { GoogleSignInButton } from "@/components/molecules/GoogleSignInButton/GoogleSignInButton";
import { useAuth } from "@/lib/firebase/auth-context";

export const LoginClient = () => {
  const auth = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (auth.state.status === "authenticated") {
      router.replace("/");
    }
  }, [auth.state.status, router]);

  return (
    <main>
      <Container>
        <Typography variant="h1">IPOtto ログイン</Typography>
        <Typography variant="body">
          Google アカウントでログインしてください。
        </Typography>
        <GoogleSignInButton />
      </Container>
    </main>
  );
};
