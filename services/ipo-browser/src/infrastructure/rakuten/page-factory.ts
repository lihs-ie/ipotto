import type { BrowserContext, Page } from "playwright";
import { chromium } from "playwright";

import {
  ImageAuthenticationPage,
  type ImageAuthenticationPagePort,
} from "./image-authentication-page.js";
import { IpoListPage, type IpoListPagePort } from "./ipo-list-page.js";
import {
  IpoApplicationPage,
  type IpoApplicationPagePort,
} from "./ipo-application-page.js";
import { LoginPage, type LoginPagePort } from "./login-page.js";

/**
 * Factory abstraction for tests and runtime.
 */
export interface PageFactoryPort {
  createSession(userDataDirectory: string): Promise<RakutenSession>;
}

/**
 * Session wrapper exposing page objects over a persistent context.
 */
export interface RakutenSession {
  /**
   * Returns the login page object.
   */
  loginPage(): LoginPagePort;

  /**
   * Returns the image authentication page object.
   */
  imageAuthenticationPage(): ImageAuthenticationPagePort;

  /**
   * Returns the IPO list page object.
   */
  ipoListPage(): IpoListPagePort;

  /**
   * Returns the IPO application page object.
   */
  ipoApplicationPage(): IpoApplicationPagePort;

  /**
   * Closes the underlying browser context.
   */
  close(): Promise<void>;
}

class PlaywrightRakutenSession implements RakutenSession {
  /**
   * Creates the session wrapper.
   */
  public constructor(
    private readonly context: BrowserContext,
    private readonly page: Page,
  ) {}

  /**
   * Returns the login page object.
   */
  public loginPage(): LoginPage {
    return new LoginPage(this.page);
  }

  /**
   * Returns the image authentication page object.
   */
  public imageAuthenticationPage(): ImageAuthenticationPage {
    return new ImageAuthenticationPage(this.page);
  }

  /**
   * Returns the IPO list page object.
   */
  public ipoListPage(): IpoListPage {
    return new IpoListPage(this.page);
  }

  /**
   * Returns the application page object.
   */
  public ipoApplicationPage(): IpoApplicationPage {
    return new IpoApplicationPage(this.page);
  }

  /**
   * Closes the browser context.
   */
  public async close(): Promise<void> {
    await this.context.close();
  }
}

/**
 * Factory for Playwright page objects.
 */
export class PageFactory implements PageFactoryPort {
  /**
   * Opens a persistent browser context and returns a page-object session.
   */
  public async createSession(userDataDirectory: string): Promise<RakutenSession> {
    const context = await chromium.launchPersistentContext(userDataDirectory, {
      headless: true,
    });
    const page = await ensurePage(context);
    return new PlaywrightRakutenSession(context, page);
  }
}

/**
 * Returns an existing page or creates one.
 */
async function ensurePage(context: BrowserContext): Promise<Page> {
  const pages = context.pages();
  if (pages.length > 0) {
    const firstPage = pages[0];
    if (firstPage !== undefined) {
      return firstPage;
    }
  }
  return context.newPage();
}
