import { expect } from "@playwright/test";
import { test } from "./fixtures/agdb.fixtures";
import { testIds } from "./test-ids";
import {
  mockDatabaseListApi,
  mockLogin,
  mockAdminUserStatus,
  mockAdminDatabaseListApi,
  gotoDbPage,
  gotoAdminDbPage,
} from "./mocks/api-helpers";

test.describe("Routing & Navigation", () => {
  test("redirects unauthenticated user to /studio/login", async ({ page }) => {
    await page.goto("/studio/db");
    // Router guard redirects to login — check for login form visibility
    // (URL may not update due to deprecated next() callback pattern)
    await expect(page.getByTestId("login_form")).toBeVisible();
  });

  test("preserves redirect query param", async ({ page }) => {
    await page.goto("/studio/db");
    await expect(page.getByTestId("login_form")).toBeVisible();
    // Check URL contains redirect param (may be /studio/login?redirect=...)
    await expect(page).toHaveURL(/.*\/studio\/login.*redirect/);
  });

  test("redirects logged-in user away from login to home", async ({
    ui,
    page,
  }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    // Navigate to login page — should redirect to /studio/db
    const response = page.waitForResponse("**/api/v1/db/list");
    await page.goto("/studio/login");
    await response;
    await ui.element(testIds.DB_TABLE).isVisible();
  });

  test("navigates to db page via nav link", async ({ ui, page }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await gotoDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();

    const navLink = page.locator('nav a[href*="/db"]');
    await expect(navLink).toBeVisible();
    await expect(navLink).toHaveText("Databases");
  });

  test("shows 404 for unknown route", async ({ page }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await page.goto("/studio/nonexistent");
    await expect(page.locator("h1")).toHaveText("404");
    await expect(page.locator("p")).toContainText("Page not found");
  });

  test("blocks non-admin from /studio/admin/db", async ({ ui, page }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    // Non-admin user navigates to admin route — should be redirected to /studio/db
    await gotoDbPage(page);
    await page.goto("/studio/admin/db");
    // Should end up at the db page (redirected by admin guard)
    await ui.element(testIds.DB_TABLE).isVisible();
  });

  test("allows admin user to access /studio/admin/db", async ({ ui, page }) => {
    await mockLogin(page);
    await mockAdminUserStatus(page);
    await mockAdminDatabaseListApi(page);
    await gotoAdminDbPage(page);
    await expect(page).toHaveURL(/.*\/studio\/admin\/db/);
    await ui.element(testIds.DB_TABLE).isVisible();
  });

  test("shows admin label on logo in admin view", async ({ page }) => {
    await mockLogin(page);
    await mockAdminUserStatus(page);
    await mockAdminDatabaseListApi(page);
    await gotoAdminDbPage(page);
    const adminLabel = page.locator(".admin-label");
    await expect(adminLabel).toBeVisible();
    await expect(adminLabel).toHaveText("admin");
  });

  test("shows Databases and Users nav links in admin view", async ({
    page,
  }) => {
    await mockLogin(page);
    await mockAdminUserStatus(page);
    await mockAdminDatabaseListApi(page);
    await gotoAdminDbPage(page);

    const navLinks = page.locator("nav a");
    await expect(navLinks).toHaveCount(2);
    await expect(navLinks.nth(0)).toHaveText("Databases");
    await expect(navLinks.nth(1)).toHaveText("Users");
  });
});
