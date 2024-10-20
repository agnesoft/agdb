import { test, expect } from "@playwright/test";

// See here how to get started:
// https://playwright.dev/docs/intro
test("visits the app root url", async ({ page }) => {
    await page.goto("/login");
    await expect(page.locator("div.login-form")).toBeVisible();
});
