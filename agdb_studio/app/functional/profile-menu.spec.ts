import { expect } from "@playwright/test";
import { test } from "./fixtures/agdb.fixtures";
import { testIds } from "./test-ids";
import {
  mockDatabaseListApi,
  mockLogin,
  mockAdminUserStatus,
  mockAdminDatabaseListApi,
  mockUserStatusWithSessions,
  mockDbActionApi,
  gotoDbPage,
  gotoAdminDbPage,
} from "./mocks/api-helpers";
import { LOGOUT_API } from "./api-paths";

test.describe("Profile Menu", () => {
  test("opens profile dropdown on click", async ({ ui, page }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await gotoDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();

    await ui.click(testIds.PROFILE_DROPDOWN);
    await expect(page.getByTestId("menu-item-change-password")).toBeVisible();
    await expect(page.getByTestId("menu-item-logout")).toBeVisible();
  });

  test("shows Change password and Logout for non-admin user", async ({
    ui,
    page,
  }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await gotoDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();

    await ui.click(testIds.PROFILE_DROPDOWN);
    await expect(page.getByTestId("menu-item-change-password")).toBeVisible();
    await expect(page.getByTestId("menu-item-logout")).toBeVisible();
    // Admin View should NOT be visible for non-admin
    await expect(page.getByTestId("menu-item-admin")).not.toBeVisible();
  });

  test("shows Admin View action for admin user", async ({ ui, page }) => {
    await mockLogin(page);
    await mockAdminUserStatus(page);
    await mockDatabaseListApi(page);
    await gotoDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();

    await ui.click(testIds.PROFILE_DROPDOWN);
    await expect(page.getByTestId("menu-item-admin")).toBeVisible();
  });

  test("shows User View action in admin view", async ({ ui, page }) => {
    await mockLogin(page);
    await mockAdminUserStatus(page);
    await mockAdminDatabaseListApi(page);
    await gotoAdminDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();

    await ui.click(testIds.PROFILE_DROPDOWN);
    await expect(page.getByTestId("menu-item-user-view")).toBeVisible();
  });

  test("navigates to admin view when Admin View clicked", async ({
    ui,
    page,
  }) => {
    await mockLogin(page);
    await mockAdminUserStatus(page);
    await mockDatabaseListApi(page);
    await mockAdminDatabaseListApi(page);
    await gotoDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();

    await ui.click(testIds.PROFILE_DROPDOWN);
    await page.getByTestId("menu-item-admin").click();
    await expect(page).toHaveURL(/.*\/studio\/admin/);
  });

  test("navigates to user view when User View clicked", async ({
    ui,
    page,
  }) => {
    await mockLogin(page);
    await mockAdminUserStatus(page);
    await mockAdminDatabaseListApi(page);
    await mockDatabaseListApi(page);
    await gotoAdminDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();

    await ui.click(testIds.PROFILE_DROPDOWN);
    await page.getByTestId("menu-item-user-view").click();
    await expect(page).toHaveURL(/.*\/studio\/db/);
  });

  test("change password: opens modal with password fields", async ({
    ui,
    page,
  }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await gotoDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();

    await ui.click(testIds.PROFILE_DROPDOWN);
    await page.getByTestId("menu-item-change-password").click();

    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();
    await expect(modal.locator(".modal-header h3")).toHaveText(
      "Change password",
    );

    // Should have 3 password inputs: current, new, confirm
    const passwordInputs = modal.locator('input[type="password"]');
    await expect(passwordInputs).toHaveCount(3);
  });

  test("change password: validates minimum length", async ({ ui, page }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await gotoDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();

    await ui.click(testIds.PROFILE_DROPDOWN);
    await page.getByTestId("menu-item-change-password").click();

    const modal = page.locator(".modal");
    const passwordInputs = modal.locator('input[type="password"]');
    // Fill current password
    await passwordInputs.nth(0).fill("oldpassword");
    // Fill new password with < 8 chars
    await passwordInputs.nth(1).fill("short");
    // Fill confirm
    await passwordInputs.nth(2).fill("short");

    await page.getByTestId("modal-button-confirm").click();

    // Modal should remain open with error message
    await expect(modal).toBeVisible();
    await expect(modal.locator(".error-message")).toContainText(
      "at least 8 characters",
    );
  });

  test("change password: validates passwords match", async ({ ui, page }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await gotoDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();

    await ui.click(testIds.PROFILE_DROPDOWN);
    await page.getByTestId("menu-item-change-password").click();

    const modal = page.locator(".modal");
    const passwordInputs = modal.locator('input[type="password"]');
    await passwordInputs.nth(0).fill("oldpassword");
    await passwordInputs.nth(1).fill("newpassword123");
    await passwordInputs.nth(2).fill("differentpassword");

    await page.getByTestId("modal-button-confirm").click();

    await expect(modal).toBeVisible();
    await expect(modal.locator(".error-message")).toContainText("do not match");
  });

  test("sessions: opens modal showing active sessions", async ({
    ui,
    page,
  }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await gotoDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();

    await ui.click(testIds.PROFILE_DROPDOWN);
    await expect(page.getByTestId("menu-item-sessions")).toBeVisible();
    await page.getByTestId("menu-item-sessions").click();

    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();
    await expect(modal.locator(".modal-header h3")).toContainText(
      "Active Sessions",
    );
    // Default mock user has no sessions
    await expect(modal).toContainText("No active sessions");
  });

  test("logout: opens confirmation modal with cluster checkbox", async ({
    ui,
    page,
  }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await gotoDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();

    await ui.click(testIds.PROFILE_DROPDOWN);
    await page.getByTestId("menu-item-logout").click();

    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();
    await expect(modal.locator(".modal-header h3")).toHaveText("Logout");
    await expect(modal).toContainText("Are you sure you want to logout?");

    // Cluster checkbox should be present
    const checkbox = modal.locator('input[type="checkbox"]');
    await expect(checkbox).toBeVisible();
    await expect(modal).toContainText("Logout from all nodes in the cluster");
  });

  test("logout: no session selector when single session", async ({
    ui,
    page,
  }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await gotoDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();

    await ui.click(testIds.PROFILE_DROPDOWN);
    await page.getByTestId("menu-item-logout").click();

    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();

    // No session select for user with 0 or 1 sessions
    const select = modal.locator("select");
    await expect(select).toHaveCount(0);
  });

  test("logout: shows session selector when multiple sessions", async ({
    ui,
    page,
  }) => {
    await mockLogin(page);
    await mockUserStatusWithSessions(page);
    await mockDatabaseListApi(page);
    await mockDbActionApi(page, LOGOUT_API);
    await gotoDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();

    await ui.click(testIds.PROFILE_DROPDOWN);
    await page.getByTestId("menu-item-logout").click();

    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();

    // Session select should be visible
    const select = modal.locator("select");
    await expect(select).toBeVisible();

    // Should have options: Current, All, Others, + 2 individual sessions
    const options = select.locator("option");
    await expect(options).toHaveCount(5);
    await expect(options.nth(0)).toHaveText("Current session");
    await expect(options.nth(1)).toHaveText("All sessions");
    await expect(options.nth(2)).toHaveText("Other sessions");

    // Cluster checkbox should still be present
    const checkbox = modal.locator('input[type="checkbox"]');
    await expect(checkbox).toBeVisible();
  });

  test("logout: can select 'Other sessions' and confirm", async ({
    ui,
    page,
  }) => {
    await mockLogin(page);
    await mockUserStatusWithSessions(page);
    await mockDatabaseListApi(page);
    await mockDbActionApi(page, LOGOUT_API);
    await gotoDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();

    await ui.click(testIds.PROFILE_DROPDOWN);
    await page.getByTestId("menu-item-logout").click();

    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();

    // Select "Other sessions"
    const select = modal.locator("select");
    await select.selectOption("others");

    // Confirm — should stay logged in (not redirect to login)
    await page.getByTestId("modal-button-confirm").click();
    await expect(modal).not.toBeVisible();
    await expect(page).not.toHaveURL(/.*\/studio\/login/);
  });
});
