import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { useMutation } from "./use-mutation";
import { err, ok, type AsyncResult } from "./result";
import { type ApiError } from "./error";

describe("useMutation", () => {
  it("starts in idle state", () => {
    const producer = async (): AsyncResult<number, ApiError> => ok(1);
    const { result } = renderHook(() => useMutation(producer));
    expect(result.current.state.status).toBe("idle");
  });

  it("reports ok state on success", async () => {
    const producer = async (input: number): AsyncResult<number, ApiError> =>
      ok(input + 1);
    const { result } = renderHook(() => useMutation(producer));
    await act(async () => {
      await result.current.mutate(10);
    });
    const state = result.current.state;
    expect(state.status).toBe("ok");
    if (state.status !== "ok") throw new Error("expected ok");
    expect(state.value).toBe(11);
  });

  it("reports error state on failure", async () => {
    const producer = async (): AsyncResult<number, ApiError> =>
      err({ kind: "network", message: "offline" });
    const { result } = renderHook(() => useMutation(producer));
    await act(async () => {
      await result.current.mutate(undefined as unknown as never);
    });
    const state = result.current.state;
    expect(state.status).toBe("error");
    if (state.status !== "error") throw new Error("expected error");
    expect(state.error.kind).toBe("network");
  });

  it("returns the Result from mutate()", async () => {
    const producer = async (): AsyncResult<string, ApiError> => ok("done");
    const { result } = renderHook(() => useMutation(producer));
    const captured = await result.current.mutate(undefined as never);
    expect(captured.ok).toBe(true);
    if (!captured.ok) throw new Error("expected ok");
    expect(captured.value).toBe("done");
  });

  it("reset() returns to idle", async () => {
    const producer = async (): AsyncResult<string, ApiError> => ok("done");
    const { result } = renderHook(() => useMutation(producer));
    await act(async () => {
      await result.current.mutate(undefined as never);
    });
    expect(result.current.state.status).toBe("ok");
    act(() => result.current.reset());
    expect(result.current.state.status).toBe("idle");
  });

  it("ignores stale results after unmount", async () => {
    let release: (value: number) => void = () => undefined;
    const pending = new Promise<number>((resolver) => {
      release = resolver;
    });
    const producer = async (): AsyncResult<number, ApiError> =>
      ok(await pending);
    const { result, unmount } = renderHook(() => useMutation(producer));
    void result.current.mutate(undefined as never);
    await waitFor(() => expect(result.current.state.status).toBe("loading"));
    unmount();
    release(42);
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(result.current.state.status).toBe("loading");
  });

  it("last mutate wins when called twice in quick succession", async () => {
    const calls: number[] = [];
    const producer = vi
      .fn()
      .mockImplementationOnce(
        async (input: number): AsyncResult<number, ApiError> => {
          calls.push(input);
          await new Promise((resolve) => setTimeout(resolve, 20));
          return ok(input);
        },
      )
      .mockImplementationOnce(
        async (input: number): AsyncResult<number, ApiError> => {
          calls.push(input);
          return ok(input);
        },
      );
    const { result } = renderHook(() => useMutation(producer));
    const firstPromise = result.current.mutate(1);
    const secondPromise = result.current.mutate(2);
    await Promise.all([firstPromise, secondPromise]);
    await waitFor(() => expect(result.current.state.status).toBe("ok"));
    const state = result.current.state;
    if (state.status !== "ok") throw new Error("expected ok");
    expect(state.value).toBe(2);
  });
});
