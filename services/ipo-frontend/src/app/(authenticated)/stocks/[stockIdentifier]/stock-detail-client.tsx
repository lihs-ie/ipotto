"use client";

import { useCallback, useMemo } from "react";
import { useParams } from "next/navigation";

import { stockIdentifierSchema } from "@ipotto/shared";

import { Typography } from "@/components/atoms/Typography/Typography";
import { StockDetailView } from "@/components/organisms/StockDetailView/StockDetailView";
import { useIpoApi } from "@/app/client-providers";
import { useAsyncResult } from "@/lib/api/use-async-result";

export const StockDetailClient = () => {
  const params = useParams<{ stockIdentifier: string }>();
  const rawIdentifier = params.stockIdentifier;
  const parsed = useMemo(
    () => stockIdentifierSchema.safeParse(rawIdentifier),
    [rawIdentifier],
  );
  const api = useIpoApi();

  const fetcher = useCallback(async () => {
    if (!parsed.success) {
      throw new Error("invalid stock identifier");
    }
    return api.getIpoStock(parsed.data);
  }, [api, parsed]);
  const { state } = useAsyncResult(fetcher, [fetcher]);

  if (!parsed.success) {
    return (
      <section>
        <Typography variant="h1">銘柄が見つかりません</Typography>
        <Typography variant="body">識別子の形式が不正です。</Typography>
      </section>
    );
  }

  if (state.status === "loading") {
    return <Typography variant="body">読み込んでいます…</Typography>;
  }

  if (state.status === "error") {
    if (state.error.kind === "http" && state.error.status === 404) {
      return (
        <section>
          <Typography variant="h1">銘柄が見つかりません</Typography>
        </section>
      );
    }
    return (
      <Typography variant="body">
        読み込みに失敗しました: {state.error.kind}
      </Typography>
    );
  }

  if (state.status === "ok") {
    return <StockDetailView stock={state.value} />;
  }

  return null;
};
