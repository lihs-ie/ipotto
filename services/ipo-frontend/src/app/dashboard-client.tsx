"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";

import { Container } from "@/components/atoms/Container/Container";
import { Typography } from "@/components/atoms/Typography/Typography";
import { useAuth } from "@/lib/firebase/auth-context";

export const DashboardClient = () => {
  const auth = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (auth.state.status === "unauthenticated") {
      router.replace("/login");
    }
  }, [auth.state.status, router]);

  if (auth.state.status !== "authenticated") {
    return (
      <main>
        <Container>
          <Typography variant="body">認証状態を確認しています…</Typography>
        </Container>
      </main>
    );
  }

  return (
    <main>
      <Container>
        <Typography variant="h1">IPOtto Dashboard</Typography>
        <Typography variant="body">ようこそ、{auth.state.user.email}</Typography>
      </Container>
    </main>
  );
};
