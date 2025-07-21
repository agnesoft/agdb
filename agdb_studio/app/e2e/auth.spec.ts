import { expect } from "@playwright/test";
import { test } from "../e2e-utils/global.setup";
import { click, fillInput } from "../e2e-utils/interaction";
import { LOGIN_API } from "../e2e-utils/apiPaths";

test("login user successfully", async ({ page }) => {
  await page.goto("/studio/login");
  await expect(page.locator("div.login-form")).toBeVisible();

  // Fill and submit the login form
  await fillInput(page, "inputUsername", "testuser");
  await fillInput(page, "inputPassword", "testpassword");
  await click(page, "buttonLogin");

  // Assert navigation or UI changes after login
  await page.goto("/studio/db");
  await expect(page).toHaveURL(/.*\/studio\/db/);
});

// test("logout user successfully", async ({ page }) => {
//   // Ensure the user is logged in first
//   await page.goto("/studio/login");
//   await fillInput(page, "inputUsername", "testuser");
//   await fillInput(page, "inputPassword", "testpassword");
//   await click(page, "buttonLogin");

//   await expect(page.locator("div.login-form")).not.toBeVisible();
//   // Navigate to the logout endpoint

//   await click(page, "profile-dropdown");
//   await click(page, "menu-item-logout");
//   await click(page, "modal-button-confirm");
//   // Assert that the user is redirected to the login page
//   await expect(page).toHaveURL(/.*\/studio\/login/);
// });

test("unsuccessful login attempt", async ({ page }) => {
  await page.route(LOGIN_API, async (route) => {
    route.fulfill({
      status: 401,
      contentType: "application/json",
      body: JSON.stringify({ error: "Invalid username or password." }),
    });
  });
  await page.goto("/studio/login");
  await expect(page.locator("div.login-form")).toBeVisible();

  // Fill and submit the login form with incorrect credentials
  await page.fill("[data-testid=inputUsername]", "wronguser");
  await page.fill("[data-testid=inputPassword]", "wrongpassword");
  await page.click("[data-testid=buttonLogin]");

  // Assert that an error message is displayed
  const errorMessage = page.locator("[data-testid=errorMessage]");
  await expect(errorMessage).toBeVisible();
  await expect(errorMessage).toHaveText("Invalid username or password.");
});
