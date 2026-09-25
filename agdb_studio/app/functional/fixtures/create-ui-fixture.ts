import { type Page } from "@playwright/test";
import { createActions } from "./actions";
import { createElementAPI } from "./element";
import { createSelectors } from "./selector";
import { type TestId } from "./test-id";

export const createUIFixture = (page: Page) => {
  const selectors = createSelectors(page);
  const actions = createActions(selectors.locator);

  return {
    element: (id: TestId) => createElementAPI(id, selectors.locator),
    ...selectors,
    ...actions,
    page,
  };
};
