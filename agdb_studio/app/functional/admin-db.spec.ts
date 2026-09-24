import { expect } from "@playwright/test";
import { test } from "./fixtures/agdb.fixtures";
import { testIds } from "./test-ids";
import {
  mockLogin,
  mockAdminUserStatus,
  mockAdminDatabaseListApi,
  gotoAdminDbPage,
} from "./mocks/api-helpers";
import { ADMIN_DB_COPY_API, ADMIN_DB_RENAME_API } from "./api-paths";

test.describe("Admin Database View", () => {
  test.beforeEach(async ({ ui, page }) => {
    await mockLogin(page);
    await mockAdminUserStatus(page);
    await mockAdminDatabaseListApi(page);
    await gotoAdminDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();
  });

  test("renders database table in admin view", async ({ page }) => {
    const rows = page.getByTestId("table-row");
    await expect(rows).toHaveCount(3);
  });

  test("admin copy action shows Owner field", async ({ page }) => {
    const row = page.getByTestId("table-row").first();
    await row.locator('button[title="Display menu"]').click();
    await page.getByTestId("menu-item-copy").click();

    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();
    // Admin copy should have both Name and Owner inputs
    const inputs = modal.locator('input[type="text"]');
    await expect(inputs).toHaveCount(2);
    // Check labels
    await expect(modal).toContainText("Name");
    await expect(modal).toContainText("Owner");
  });

  test("admin rename action shows Owner field", async ({ page }) => {
    const row = page.getByTestId("table-row").first();
    await row.locator('button[title="Display menu"]').click();
    await page.getByTestId("menu-item-rename").click();

    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();
    // Admin rename should have both Name and Owner inputs
    const inputs = modal.locator('input[type="text"]');
    await expect(inputs).toHaveCount(2);
    await expect(modal).toContainText("Name");
    await expect(modal).toContainText("Owner");
  });

  test("uses admin API endpoint for copy", async ({ page }) => {
    let adminApiCalled = false;
    await page.route(ADMIN_DB_COPY_API, async (route) => {
      adminApiCalled = true;
      route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({}),
      });
    });
    await mockAdminDatabaseListApi(page);

    const row = page.getByTestId("table-row").first();
    await row.locator('button[title="Display menu"]').click();
    await page.getByTestId("menu-item-copy").click();

    const modal = page.locator(".modal");
    const nameInput = modal.locator('input[type="text"]').first();
    await nameInput.fill("copied_db");
    await page.getByTestId("modal-button-confirm").click();

    expect(adminApiCalled).toBe(true);
  });
});
