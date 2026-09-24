import { test, expect } from "./fixtures/agdb.fixtures";
import { testIds, dynamicIds } from "./test-ids";
import { LOGIN_API } from "./api-paths";

test.describe("Authentication", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/studio/login");
  });

  test("login user successfully", async ({ ui, page }) => {
    await ui.element(testIds.LOGIN_FORM).isVisible();

    await ui.fill(testIds.INPUT_USERNAME, "testuser");
    await ui.fill(testIds.INPUT_PASSWORD, "testpassword");
    await ui.click(testIds.BUTTON_LOGIN);

    // Assert navigation or UI changes after login
    await page.goto("/studio/db");
    await expect(page).toHaveURL(/.*\/studio\/db/);
    await ui.element(testIds.LOGIN_FORM).isHidden();
  });

  test("logout user successfully", async ({ ui, page }) => {
    // Ensure the user is logged in first
    await ui.fill(testIds.INPUT_USERNAME, "testuser");
    await ui.fill(testIds.INPUT_PASSWORD, "testpassword");
    await ui.click(testIds.BUTTON_LOGIN);

    await ui.element(testIds.LOGIN_FORM).isHidden();

    await ui.click(testIds.PROFILE_DROPDOWN);
    await ui.click(dynamicIds.menuItem("logout"));
    await ui.click(dynamicIds.modalButton("confirm"));

    // Assert that the user is redirected to the login page
    await expect(page).toHaveURL(/.*\/studio\/login/);
  });

  test("unsuccessful login attempt", async ({ ui, page }) => {
    await page.route(LOGIN_API, async (route) => {
      route.fulfill({
        status: 401,
        contentType: "application/json",
        body: JSON.stringify({ error: "Invalid username or password." }),
      });
    });

    // Fill and submit the login form with incorrect credentials
    await ui.fill(testIds.INPUT_USERNAME, "wronguser");
    await ui.fill(testIds.INPUT_PASSWORD, "wrongpassword");
    await ui.click(testIds.BUTTON_LOGIN);

    // Assert that an error message is displayed
    await ui
      .element(testIds.ERROR_MESSAGE)
      .hasText("Invalid username or password.");
  });
});
