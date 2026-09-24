import { test as base } from "@playwright/test";
import { createUIFixture } from "./create-ui-fixture";

type UIFixtures = {
  ui: ReturnType<typeof createUIFixture>;
};

export const test = base.extend<UIFixtures>({
  ui: async ({ page }, use) => {
    await use(createUIFixture(page));
  },
});

export { expect } from "@playwright/test";
