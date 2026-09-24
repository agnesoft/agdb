import type { PlaywrightTestConfig } from "@playwright/test";
import { devices } from "@playwright/test";

/**
 * Read environment variables from file.
 * https://github.com/motdotla/dotenv
 */
// require('dotenv').config();

/**
 * See https://playwright.dev/docs/test-configuration.
 */
const config: PlaywrightTestConfig = {
  testDir: "./functional",
  /* Maximum time one test can run for. */
  timeout: 30 * 1000,
  expect: {
    /**
     * Maximum time expect() should wait for the condition to be met.
     * SPA needs time to: load JS → fetch OpenAPI → init client → check auth → fetch data → render.
     */
    timeout: 15000,
  },
  /* Run tests in files in parallel */
  fullyParallel: true,
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* Retry on CI only */
  retries: process.env.CI ? 2 : 0,
  /* Opt out of parallel tests on CI. */
  workers: process.env.CI ? 4 : undefined,
  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: [["line"], ["junit"], ["html", { open: "never" }]],
  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
  use: {
    /* Maximum time each action such as `click()` can take. Defaults to 0 (no limit). */
    actionTimeout: 0,
    /* Base URL to use in actions like `await page.goto('/')`. */
    baseURL: "http://localhost:5173",

    ignoreHTTPSErrors: true,
    timezoneId: "Europe/London",
    locale: "en-GB",

    /* Only on CI systems run the tests headless */
    headless: !!process.env.CI,

    screenshot: "on-first-failure" as const,
    trace: process.env.CI
      ? ("on-first-retry" as const)
      : ("retain-on-failure" as const),
    video: process.env.CI ? ("on-first-retry" as const) : ("off" as const),
    launchOptions: {
      chromiumSandbox: false,
      args: [
        "--headless=new",
        "--no-sandbox",
        "--ignore-certificate-errors",
        "--disable-gpu",
        "--disable-dev-shm-usage",
        "--disable-extensions",
        "--disable-background-timer-throttling",
        "--disable-setuid-sandbox",
      ],
    },
  },

  /* Configure projects for major browsers */
  projects: [
    {
      name: "chromium",
      testMatch: "**/*.spec.ts",
      use: {
        ...devices["Desktop Chrome"],
      },
    },
    // {
    //     name: "firefox",
    //     use: {
    //         ...devices["Desktop Firefox"],
    //     },
    // },
    // {
    //     name: "webkit",
    //     use: {
    //         ...devices["Desktop Safari"],
    //     },
    // },

    // /* Test against mobile viewports. */
    // {
    //     name: "Mobile Chrome",
    //     use: {
    //         ...devices["Pixel 5"],
    //     },
    // },
    // {
    //     name: "Mobile Safari",
    //     use: {
    //         ...devices["iPhone 12"],
    //     },
    // },

    /* Test against branded browsers. */
    // {
    //   name: 'Microsoft Edge',
    //   use: {
    //     channel: 'msedge',
    //   },
    // },
    // {
    //   name: 'Google Chrome',
    //   use: {
    //     channel: 'chrome',
    //   },
    // },
  ],

  /* Folder for test artifacts such as screenshots, videos, traces, etc. */
  // outputDir: 'test-results/',

  /* Run your local dev server before starting the tests */
  webServer: {
    /**
     * Use the dev server by default for faster feedback loop.
     * Use the preview server on CI for more realistic testing.
    Playwright will re-use the local server if there is already a dev-server running.
     */
    command: process.env.CI ? "vite preview --port 5173" : "vite dev",
    port: 5173,
    reuseExistingServer: !process.env.CI,
  },
};

export default config;
