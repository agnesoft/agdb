import { expect } from "@playwright/test";
import { test } from "./fixtures/agdb.fixtures";
import { testIds, dynamicIds } from "./test-ids";
import {
  mockDatabaseListApi,
  mockLogin,
  mockAdminUserStatus,
  mockAdminDatabaseListApi,
  gotoDbPage,
  gotoAdminDbPage,
} from "./mocks/api-helpers";
import { USER_CHANGE_PASSWORD_API } from "./api-paths";

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
});
