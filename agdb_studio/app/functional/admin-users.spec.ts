import { expect } from "@playwright/test";
import { test } from "./fixtures/agdb.fixtures";
import {
  mockLogin,
  mockAdminUserStatus,
  mockAdminDatabaseListApi,
  mockAdminUserListApi,
  mockDbActionApi,
  gotoAdminUsersPage,
} from "./mocks/api-helpers";
import {
  ADMIN_USER_ADD_API,
  ADMIN_USER_DELETE_API,
  ADMIN_USER_CHANGE_PASSWORD_API,
  ADMIN_USER_LOGOUT_API,
  ADMIN_USER_LIST_API,
} from "./api-paths";

test.describe("Admin User Management", () => {
  test.beforeEach(async ({ page }) => {
    await mockLogin(page);
    await mockAdminUserStatus(page);
    await mockAdminDatabaseListApi(page);
    await mockAdminUserListApi(page);
    await gotoAdminUsersPage(page);
  });

  test("displays user table with correct data", async ({ page }) => {
    const rows = page.getByTestId("table-row");
    await expect(rows).toHaveCount(3);

    // First user: admin
    const firstRow = rows.nth(0);
    await expect(firstRow.getByTestId("table-cell-username")).toContainText(
      "admin",
    );
    await expect(firstRow.getByTestId("table-cell-sessionCount")).toBeVisible();

    // Second user: inactive (sorted alphabetically)
    const secondRow = rows.nth(1);
    await expect(secondRow.getByTestId("table-cell-username")).toContainText(
      "inactive",
    );

    // Third user: testuser
    const thirdRow = rows.nth(2);
    await expect(thirdRow.getByTestId("table-cell-username")).toContainText(
      "testuser",
    );
  });

  test("shows crown icon next to admin users", async ({ page }) => {
    const rows = page.getByTestId("table-row");
    const adminRow = rows.nth(0);
    // Admin user should have crown icon
    const crownIcon = adminRow.locator(".crown-icon");
    await expect(crownIcon).toBeVisible();

    // Non-admin user should NOT have crown icon (testuser at index 2)
    const userRow = rows.nth(2);
    const noIcon = userRow.locator(".crown-icon");
    await expect(noIcon).toHaveCount(0);
  });

  test("adds new user via form", async ({ page }) => {
    await mockDbActionApi(page, ADMIN_USER_ADD_API);
    await mockAdminUserListApi(page);

    // Fill in the add user form
    await page.locator("#username").fill("newuser");
    await page.locator("#password").fill("newpassword");
    await page.locator('button[type="submit"]').click();
  });

  test("delete user: opens confirmation and confirms", async ({ page }) => {
    await mockDbActionApi(page, ADMIN_USER_DELETE_API);
    await mockAdminUserListApi(page);

    // Open action menu on a non-admin user row
    const row = page.getByTestId("table-row").nth(1);
    await row.locator('button[title="Display menu"]').click();
    await page.getByTestId("menu-item-delete").click();

    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();
    await expect(modal).toContainText("delete the user");
    await page.getByTestId("modal-button-confirm").click();
    await expect(modal).not.toBeVisible();
  });

  test("change password: opens modal with password input", async ({ page }) => {
    await mockDbActionApi(page, ADMIN_USER_CHANGE_PASSWORD_API);

    const row = page.getByTestId("table-row").nth(1);
    await row.locator('button[title="Display menu"]').click();
    await page.getByTestId("menu-item-change_password").click();

    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();
    await expect(modal.locator(".modal-header h3")).toContainText(
      "Change password",
    );
    const passwordInput = modal.locator('input[type="text"]');
    await expect(passwordInput).toBeVisible();

    await passwordInput.fill("newpassword123");
    await page.getByTestId("modal-button-confirm").click();
    await expect(modal).not.toBeVisible();
  });

  test("logout user: opens confirmation with cluster checkbox", async ({
    page,
  }) => {
    await mockDbActionApi(page, ADMIN_USER_LOGOUT_API);

    const row = page.getByTestId("table-row").nth(1);
    await row.locator('button[title="Display menu"]').click();
    await page.getByTestId("menu-item-logout").click();

    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();
    await expect(modal).toContainText("log out the user");
    const checkbox = modal.locator('input[type="checkbox"]');
    await expect(checkbox).toBeVisible();
  });

  test("displays session count per user", async ({ page }) => {
    const rows = page.getByTestId("table-row");

    // admin: 2 sessions
    await expect(
      rows.nth(0).getByTestId("table-cell-sessionCount"),
    ).toContainText("2");

    // inactive: 0 sessions
    await expect(
      rows.nth(1).getByTestId("table-cell-sessionCount"),
    ).toContainText("0");

    // testuser: 1 session
    await expect(
      rows.nth(2).getByTestId("table-cell-sessionCount"),
    ).toContainText("1");
  });

  test("view sessions: opens modal with session table", async ({ page }) => {
    const row = page.getByTestId("table-row").nth(2); // testuser
    await row.locator('button[title="Display menu"]').click();
    await page.getByTestId("menu-item-view_sessions").click();

    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();
    await expect(modal.locator(".modal-header h3")).toContainText(
      "Sessions for testuser",
    );
    // Table with session data
    await expect(modal.locator("table")).toBeVisible();
    await expect(modal.locator("th")).toContainText([
      "Agent",
      "Created",
      "Expires",
    ]);
    await expect(modal.locator("td").first()).toContainText("Safari");
  });

  test("view sessions: shows empty message for user with no sessions", async ({
    page,
  }) => {
    const row = page.getByTestId("table-row").nth(1); // inactive
    await row.locator('button[title="Display menu"]').click();
    await page.getByTestId("menu-item-view_sessions").click();

    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();
    await expect(modal).toContainText("No active sessions");
  });

  test("logout user with multiple sessions: shows session select", async ({
    page,
  }) => {
    await mockDbActionApi(page, ADMIN_USER_LOGOUT_API);

    const row = page.getByTestId("table-row").nth(0); // admin (2 sessions)
    await row.locator('button[title="Display menu"]').click();
    await page.getByTestId("menu-item-logout").click();

    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();
    await expect(modal).toContainText("log out the user");

    // Session select should be visible for multi-session user
    const select = modal.locator("select");
    await expect(select).toBeVisible();

    // Cluster checkbox should still be present
    const checkbox = modal.locator('input[type="checkbox"]');
    await expect(checkbox).toBeVisible();
  });

  test("handles empty user list", async ({ page }) => {
    await mockAdminUserListApi(page, []);
    await page.reload();
    await expect(page.getByTestId("table-row")).toHaveCount(0);
  });

  test("handles API error on user list fetch", async ({ page }) => {
    await page.route(ADMIN_USER_LIST_API, async (route) => {
      route.fulfill({
        status: 500,
        contentType: "application/json",
        body: JSON.stringify({ error: "Internal Server Error" }),
      });
    });
    const response = page.waitForResponse(ADMIN_USER_LIST_API);
    await page.reload();
    await response;
    // Open the notification viewer to see the error
    await page.locator(".notification-button").click();
    await expect(page.locator(".notification-viewer")).toBeVisible();
    await expect(page.getByTestId("notification-item")).toContainText(
      "Internal Server Error",
    );
  });
});
