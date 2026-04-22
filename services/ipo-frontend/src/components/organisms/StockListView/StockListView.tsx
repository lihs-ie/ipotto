"use client";

import { useCallback, useState } from "react";
import { useRouter } from "next/navigation";

import {
  type StockIdentifier,
  type StockStatus,
} from "@ipotto/shared";

import { Typography } from "@/components/atoms/Typography/Typography";
import { StockStatusFilter } from "@/components/molecules/StockStatusFilter/StockStatusFilter";
import { StockTable } from "@/components/molecules/StockTable/StockTable";
import { useIpoApi } from "@/app/client-providers";
import { useAsyncResult } from "@/lib/api/use-async-result";

export const StockListView = () => {
  const api = useIpoApi();
  const router = useRouter();
  const [filter, setFilter] = useState<StockStatus | null>(null);

  const fetcher = useCallback(
    () => api.listIpoStocks(filter === null ? {} : { status: filter }),
    [api, filter],
  );
  const { state } = useAsyncResult(fetcher, [fetcher]);

  const handleRowClick = useCallback(
    (identifier: StockIdentifier) => {
      router.push(`/stocks/${encodeURIComponent(identifier)}`);
    },
    [router],
  );

  return (
    <section>
      <Typography variant="h1">銘柄一覧</Typography>
      <StockStatusFilter value={filter} onChange={setFilter} />
      {state.status === "loading" && (
        <Typography variant="body">読み込んでいます…</Typography>
      )}
      {state.status === "error" && (
        <Typography variant="body">
          読み込みに失敗しました: {state.error.kind}
        </Typography>
      )}
      {state.status === "ok" && (
        <>
          <Typography variant="caption">
            全 {state.value.totalCount} 件
          </Typography>
          <StockTable stocks={state.value.items} onRowClick={handleRowClick} />
        </>
      )}
    </section>
  );
};
