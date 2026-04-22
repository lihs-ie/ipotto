"use client";

import { usePathname } from "next/navigation";
import { type ReactNode } from "react";

import { Footer } from "@/components/organisms/Footer/Footer";
import { Header } from "@/components/organisms/Header/Header";
import { Sidebar } from "@/components/organisms/Sidebar/Sidebar";
import { useAuth } from "@/lib/firebase/auth-context";

import styles from "./AppShell.module.css";

type Props = {
  children: ReactNode;
};

export const AppShell = (props: Props) => {
  const auth = useAuth();
  const pathname = usePathname() ?? "/";
  const userEmail =
    auth.state.status === "authenticated" ? auth.state.user.email : null;

  return (
    <div className={styles.container}>
      <Header userEmail={userEmail} onSignOut={auth.signOut} />
      <Sidebar currentPath={pathname} />
      <main className={styles.main}>{props.children}</main>
      <Footer />
    </div>
  );
};
