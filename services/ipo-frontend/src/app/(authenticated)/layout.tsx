"use client";

import { useRouter } from "next/navigation";
import { useEffect, type ReactNode } from "react";

import { Container } from "@/components/atoms/Container/Container";
import { Typography } from "@/components/atoms/Typography/Typography";
import { AppShell } from "@/components/templates/AppShell/AppShell";
import { useAuth } from "@/lib/firebase/auth-context";

type Props = {
  children: ReactNode;
};

export default function AuthenticatedLayout(props: Props) {
  const auth = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (auth.state.status === "unauthenticated") {
      router.replace("/login");
    }
  }, [auth.state.status, router]);

  if (auth.state.status === "loading") {
    return (
      <Container>
        <Typography variant="body">認証状態を確認しています…</Typography>
      </Container>
    );
  }

  if (auth.state.status === "unauthenticated") {
    return (
      <Container>
        <Typography variant="body">ログインページへ移動しています…</Typography>
      </Container>
    );
  }

  return <AppShell>{props.children}</AppShell>;
}
