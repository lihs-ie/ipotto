"use client";

import { useCallback, useEffect, useState } from "react";

import {
  type ListOperationLogsQuery,
  type OperationLogSummary,
} from "@ipotto/shared";

import { Typography } from "@/components/atoms/Typography/Typography";
import {
  OperationLogFilter,
  type OperationLogFilterValue,
} from "@/components/molecules/OperationLogFilter/OperationLogFilter";
import { OperationLogTable } from "@/components/molecules/OperationLogTable/OperationLogTable";
import { Pagination } from "@/components/molecules/Pagination/Pagination";
import { useIpoApi } from "@/app/client-providers";

export const OperationLogView = () => {
  const api = useIpoApi();
  const [filter, setFilter] = useState<OperationLogFilterValue>({
    eventType: null,
  });
  const [items, setItems] = useState<OperationLogSummary[]>([]);
  const [cursor, setCursor] = useState<string | null>(null);
  const [hasMore, setHasMore] = useState(false);
  const [status, setStatus] = useState<"idle" | "loading" | "error">("idle");

  const buildQuery = useCallback(
    (nextCursor: string | null): ListOperationLogsQuery => {
      const query: ListOperationLogsQuery = {};
      if (filter.eventType !== null) query.eventType = filter.eventType;
      if (nextCursor !== null) query.cursor = nextCursor;
      return query;
    },
    [filter],
  );

  const fetchPage = useCallback(
    async (nextCursor: string | null): Promise<void> => {
      setStatus("loading");
      const result = await api.listOperationLogs(buildQuery(nextCursor));
      if (!result.ok) {
        setStatus("error");
        return;
      }
      setItems((previous) =>
        nextCursor === null
          ? result.value.items
          : previous.concat(result.value.items),
      );
      setCursor(result.value.nextCursor);
      setHasMore(result.value.hasMore);
      setStatus("idle");
    },
    [api, buildQuery],
  );

  useEffect(() => {
    setItems([]);
    setCursor(null);
    void fetchPage(null);
  }, [fetchPage]);

  const handleNext = useCallback((): void => {
    if (cursor === null) return;
    void fetchPage(cursor);
  }, [cursor, fetchPage]);

  return (
    <section>
      <Typography variant="h1">操作ログ</Typography>
      <OperationLogFilter value={filter} onChange={setFilter} />
      {status === "error" && (
        <Typography variant="body">読み込みに失敗しました。</Typography>
      )}
      <OperationLogTable items={items} />
      <Pagination
        hasMore={hasMore}
        onNext={handleNext}
        loading={status === "loading"}
      />
    </section>
  );
};
