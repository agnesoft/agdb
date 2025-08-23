import { type Page } from "@playwright/test";

export const takeScreenshot = async (
  page: Page,
  name: string,
  _testName?: string,
): Promise<void> => {
  const testName = _testName || "screenshot";
  const path = `./screenshots/${testName}/${name.replace(/[^a-z0-9]/gi, "_").toLowerCase()}.png`;
  await page.context().storageState({ path });
  await page.screenshot({ path });
};
