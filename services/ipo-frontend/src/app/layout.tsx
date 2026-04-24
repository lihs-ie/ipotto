import type { Metadata } from "next";
import "@/styles/globals.css";

import { ClientProviders } from "./client-providers";

export const metadata: Metadata = {
  title: "IPOtto",
  description: "IPO抽選申し込み自動化・管理",
};

type Props = {
  children: React.ReactNode;
};

export default function RootLayout(props: Props) {
  return (
    <html lang="ja">
      <body>
        <ClientProviders>{props.children}</ClientProviders>
      </body>
    </html>
  );
}
