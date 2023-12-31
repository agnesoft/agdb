import { test, expect } from "@playwright/test";

// See here how to get started:
// https://playwright.dev/docs/intro
test("visits the app root url", async ({ page, baseURL }) => {
    await page.goto(baseURL + "/");
    await expect(page.locator("main")).toContainText("agdb");
});
