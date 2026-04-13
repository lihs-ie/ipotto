import { afterEach, describe, expect, it, vi } from "vitest";

describe("index bootstrap", () => {
  afterEach(() => {
    vi.resetModules();
    vi.restoreAllMocks();
    vi.doUnmock("./infrastructure/config/app-config.js");
    vi.doUnmock("./server.js");
  });

  it("reads config, creates the app, and starts listening", async () => {
    const listen = vi.fn((_port: number, _host: string, callback?: () => void) => {
      callback?.();
      return {};
    });
    const createApp = vi.fn().mockReturnValue({ listen });
    const readAppConfig = vi.fn().mockReturnValue({ port: 8081 });
    const consoleSpy = vi.spyOn(console, "log").mockImplementation(() => undefined);

    vi.doMock("./infrastructure/config/app-config.js", () => ({
      readAppConfig,
    }));
    vi.doMock("./server.js", () => ({
      createApp,
    }));

    await import("./index.js");

    expect(readAppConfig).toHaveBeenCalledTimes(1);
    expect(createApp).toHaveBeenCalledTimes(1);
    expect(listen).toHaveBeenCalledWith(8081, "0.0.0.0", expect.any(Function));
    expect(consoleSpy).toHaveBeenCalledWith("ipo-browser listening on port 8081");
  });
});
