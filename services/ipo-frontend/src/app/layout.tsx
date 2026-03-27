import type { Metadata } from "next";
import "@/styles/globals.css";

export const metadata: Metadata = {
  title: "IPOtto",
  description: "IPO抽選申し込み自動化・管理",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="ja">
      <body>{children}</body>
    </html>
  );
}
