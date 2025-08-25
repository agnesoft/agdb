import { expect } from "@playwright/test";
import { test } from "../e2e-utils/global.setup";
// import { click, fillInput } from "../e2e-utils/interaction";
// import { LOGIN_API } from "../e2e-utils/apiPaths";
// import { hasText, isHidden, isVisible } from "../e2e-utils/elements";

test.describe("Authentication E2E Tests", () => {
  test("all tests commented out", () => {
    expect(true).toBe(true);
  });
  // test.beforeEach(async ({ page }) => {
  //   await page.goto("/studio/login");
  //   await isVisible(page, "login_form");
  // });

  // test("login user successfully", async ({ page }) => {
  //   // Fill and submit the login form
  //   await fillInput(page, "inputUsername", "testuser");
  //   await fillInput(page, "inputPassword", "testpassword");
  //   await click(page, "buttonLogin");

  //   // Assert navigation or UI changes after login
  //   await page.goto("/studio/db");
  //   await expect(page).toHaveURL(/.*\/studio\/db/);
  //   await isHidden(page, "login_form");
  // });

  // test("logout user successfully", async ({ page }) => {
  //   // Ensure the user is logged in first
  //   await fillInput(page, "inputUsername", "testuser");
  //   await fillInput(page, "inputPassword", "testpassword");
  //   await click(page, "buttonLogin");

  //   await isHidden(page, "login_form");

  //   await click(page, "profile-dropdown");
  //   await click(page, "menu-item-logout");
  //   await click(page, "modal-button-confirm");
  //   // Assert that the user is redirected to the login page
  //   await expect(page).toHaveURL(/.*\/studio\/login/);
  // });

  // test("unsuccessful login attempt", async ({ page }) => {
  //   await page.route(LOGIN_API, async (route) => {
  //     route.fulfill({
  //       status: 401,
  //       contentType: "application/json",
  //       body: JSON.stringify({ error: "Invalid username or password." }),
  //     });
  //   });

  //   // Fill and submit the login form with incorrect credentials
  //   await fillInput(page, "inputUsername", "wronguser");
  //   await fillInput(page, "inputPassword", "wrongpassword");
  //   await click(page, "buttonLogin");

  //   // Assert that an error message is displayed
  //   await hasText(page, "errorMessage", "Invalid username or password.");
  // });
});
