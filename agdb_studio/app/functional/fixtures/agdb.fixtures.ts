/* eslint-disable react-hooks/rules-of-hooks */
import { test as base } from "./base.fixtures";
import { enableConsoleLogging, mockAllApis } from "../mocks/api-helpers";

type AgdbFixtures = {
  _apiSetup: void;
};

export const test = base.extend<AgdbFixtures>({
  _apiSetup: [
    async ({ page }, use) => {
      await mockAllApis(page);
      enableConsoleLogging(page);
      await use();
    },
    { auto: true },
  ],
});

export { expect } from "@playwright/test";
