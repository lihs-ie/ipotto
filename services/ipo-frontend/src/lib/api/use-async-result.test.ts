import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { useAsyncResult } from "./use-async-result";
import { err, ok, type AsyncResult } from "./result";
import { type ApiError } from "./error";

const flush = async (): Promise<void> => {
  await new Promise((resolve) => setTimeout(resolve, 0));
};

describe("useAsyncResult", () => {
  it("reports loading → ok for a successful producer", async () => {
    const producer = async (): AsyncResult<number, ApiError> => ok(42);
    const { result } = renderHook(() => useAsyncResult(producer, []));
    expect(result.current.state.status).toBe("loading");
    await waitFor(() => expect(result.current.state.status).toBe("ok"));
    const finalState = result.current.state;
    if (finalState.status !== "ok") throw new Error("expected ok");
    expect(finalState.value).toBe(42);
  });

  it("reports error for a failing producer", async () => {
    const producer = async (): AsyncResult<number, ApiError> =>
      err({ kind: "network", message: "offline" });
    const { result } = renderHook(() => useAsyncResult(producer, []));
    await waitFor(() => expect(result.current.state.status).toBe("error"));
    const finalState = result.current.state;
    if (finalState.status !== "error") throw new Error("expected error");
    expect(finalState.error.kind).toBe("network");
  });

  it("re-runs producer when deps change", async () => {
    const producer = vi.fn(
      async (): AsyncResult<string, ApiError> => ok("value"),
    );
    let dep = 1;
    const { result, rerender } = renderHook(() =>
      useAsyncResult(producer, [dep]),
    );
    await waitFor(() => expect(result.current.state.status).toBe("ok"));
    expect(producer).toHaveBeenCalledTimes(1);

    dep = 2;
    rerender();
    await waitFor(() => expect(producer).toHaveBeenCalledTimes(2));
  });

  it("reload() re-runs the producer on demand", async () => {
    const producer = vi.fn(
      async (): AsyncResult<string, ApiError> => ok("value"),
    );
    const { result } = renderHook(() => useAsyncResult(producer, []));
    await waitFor(() => expect(result.current.state.status).toBe("ok"));
    expect(producer).toHaveBeenCalledTimes(1);

    act(() => {
      result.current.reload();
    });
    await waitFor(() => expect(producer).toHaveBeenCalledTimes(2));
  });

  it("ignores stale results after unmount", async () => {
    let release: ((value: number) => void) = () => undefined;
    const pending = new Promise<number>((resolver) => {
      release = resolver;
    });
    const producer = async (): AsyncResult<number, ApiError> =>
      ok(await pending);
    const { result, unmount } = renderHook(() => useAsyncResult(producer, []));
    expect(result.current.state.status).toBe("loading");
    unmount();
    release(10);
    await flush();
    expect(result.current.state.status).toBe("loading");
  });
});
