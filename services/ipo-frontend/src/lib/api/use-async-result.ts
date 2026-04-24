"use client";

import { useCallback, useEffect, useRef, useState } from "react";

import { type ApiError } from "./error";
import { type AsyncResult } from "./result";

export type AsyncResultState<T, E> =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "ok"; value: T }
  | { status: "error"; error: E };

export type UseAsyncResult<T, E> = {
  state: AsyncResultState<T, E>;
  reload: () => void;
};

/// React hook that runs an `AsyncResult<T, E>` producer on mount and
/// whenever `deps` change. Automatically cancels pending updates when
/// the component unmounts or when a new fetch starts, so stale
/// responses never overwrite fresh state. `reload()` re-runs the
/// producer without needing to change `deps`.
///
/// Defaults `E` to `ApiError` for convenience — most callers consume
/// it through `useIpoApi()` which produces `AsyncResult<T, ApiError>`.
export const useAsyncResult = <T, E = ApiError>(
  producer: () => AsyncResult<T, E>,
  deps: unknown[],
): UseAsyncResult<T, E> => {
  const [state, setState] = useState<AsyncResultState<T, E>>({
    status: "idle",
  });
  const reloadCounterRef = useRef(0);

  const run = useCallback(async (token: number): Promise<void> => {
    setState({ status: "loading" });
    const result = await producer();
    if (token !== reloadCounterRef.current) return;
    if (result.ok) {
      setState({ status: "ok", value: result.value });
    } else {
      setState({ status: "error", error: result.error });
    }
  }, deps);

  useEffect(() => {
    reloadCounterRef.current += 1;
    const token = reloadCounterRef.current;
    void run(token);
    return () => {
      reloadCounterRef.current += 1;
    };
  }, [run]);

  const reload = useCallback((): void => {
    reloadCounterRef.current += 1;
    const token = reloadCounterRef.current;
    void run(token);
  }, [run]);

  return { state, reload };
};
