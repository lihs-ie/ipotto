"use client";

import { useCallback, useRef, useState } from "react";

import { type ApiError } from "./error";
import { type AsyncResult, type Result, err, ok } from "./result";

export type MutationState<T, E> =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "ok"; value: T }
  | { status: "error"; error: E };

export type UseMutation<Input, T, E> = {
  state: MutationState<T, E>;
  mutate: (input: Input) => Promise<Result<T, E>>;
  reset: () => void;
};

/// React hook for POST / PUT / DELETE style operations. Tracks the
/// most recent attempt in `state`, returns the typed `Result` to the
/// caller so UI code can decide what to do next (e.g. close a dialog
/// / reload a list), and skips stale updates when the component
/// unmounts mid-flight.
export const useMutation = <Input, T, E = ApiError>(
  producer: (input: Input) => AsyncResult<T, E>,
): UseMutation<Input, T, E> => {
  const [state, setState] = useState<MutationState<T, E>>({ status: "idle" });
  const tokenRef = useRef(0);

  const mutate = useCallback(
    async (input: Input): Promise<Result<T, E>> => {
      tokenRef.current += 1;
      const token = tokenRef.current;
      setState({ status: "loading" });
      const result = await producer(input);
      if (token !== tokenRef.current) return result;
      if (result.ok) {
        setState({ status: "ok", value: result.value });
        return ok(result.value);
      }
      setState({ status: "error", error: result.error });
      return err(result.error);
    },
    [producer],
  );

  const reset = useCallback((): void => {
    tokenRef.current += 1;
    setState({ status: "idle" });
  }, []);

  return { state, mutate, reset };
};
