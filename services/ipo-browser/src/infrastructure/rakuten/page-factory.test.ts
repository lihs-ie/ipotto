import { beforeEach, describe, expect, it, vi } from "vitest";

const launchPersistentContext = vi.fn();

vi.mock("playwright", () => ({
  chromium: {
    launchPersistentContext,
  },
}));

describe("PageFactory", () => {
  beforeEach(() => {
    vi.resetModules();
    launchPersistentContext.mockReset();
  });

  it("reuses the first existing page in a persistent context", async () => {
    const page = {};
    const context = {
      pages: vi.fn().mockReturnValue([page]),
      newPage: vi.fn(),
      close: vi.fn().mockResolvedValue(undefined),
    };
    launchPersistentContext.mockResolvedValue(context);

    const { PageFactory } = await import("./page-factory.js");
    const factory = new PageFactory();
    const session = await factory.createSession("/tmp/browser-session");

    expect(launchPersistentContext).toHaveBeenCalledWith("/tmp/browser-session", {
      headless: true,
    });
    expect(context.newPage).not.toHaveBeenCalled();
    expect(typeof session.loginPage().enterLoginId).toBe("function");

    await session.close();
    expect(context.close).toHaveBeenCalled();
  });

  it("creates a new page when the context has none", async () => {
    const page = {};
    const context = {
      pages: vi.fn().mockReturnValue([]),
      newPage: vi.fn().mockResolvedValue(page),
      close: vi.fn().mockResolvedValue(undefined),
    };
    launchPersistentContext.mockResolvedValue(context);

    const { PageFactory } = await import("./page-factory.js");
    const factory = new PageFactory();

    await factory.createSession("/tmp/browser-session");

    expect(context.newPage).toHaveBeenCalledTimes(1);
  });
});
