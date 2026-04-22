"use client";

import { useCallback } from "react";

import { Typography } from "@/components/atoms/Typography/Typography";
import { DashboardOverview } from "@/components/organisms/DashboardOverview/DashboardOverview";
import { useIpoApi } from "@/app/client-providers";
import { useAsyncResult } from "@/lib/api/use-async-result";

export const DashboardClient = () => {
  const api = useIpoApi();
  const fetcher = useCallback(() => api.getDashboardSummary(), [api]);
  const { state } = useAsyncResult(fetcher, [fetcher]);

  return (
    <section>
      <Typography variant="h1">ダッシュボード</Typography>
      {state.status === "loading" && (
        <Typography variant="body">読み込んでいます…</Typography>
      )}
      {state.status === "error" && (
        <Typography variant="body">
          読み込みに失敗しました: {state.error.kind}
        </Typography>
      )}
      {state.status === "ok" && <DashboardOverview summary={state.value} />}
    </section>
  );
};
