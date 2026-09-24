import { type Locator, type Page } from "@playwright/test";
import { type TestId } from "./test-id";

export const createSelectors = (page: Page) => {
  const locator = (testId: TestId): Locator => page.getByTestId(testId);

  return {
    locator,
  };
};
