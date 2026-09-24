import { expect } from "@playwright/test";
import { test } from "./fixtures/agdb.fixtures";
import { testIds, dynamicIds } from "./test-ids";
import {
  mockDatabaseListApi,
  mockLogin,
  mockDbActionApi,
  gotoDbPage,
} from "./mocks/api-helpers";
import { DB_BACKUP_API, DB_COPY_API, DB_DELETE_API } from "./api-paths";

test.describe("Modal System", () => {
  test.beforeEach(async ({ ui, page }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await gotoDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();
  });

  test("modal opens with header and content", async ({ page }) => {
    await mockDbActionApi(page, DB_DELETE_API);
    // Open actions menu on first row
    const row = page.getByTestId("table-row").first();
    await row.locator('button[title="Display menu"]').click();
    // Click "Delete" action
    await page.getByTestId("menu-item-delete").click();
    // Modal should be visible with a header
    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();
    const header = modal.locator(".modal-header h3");
    await expect(header).toContainText("Confirm action for");
  });

  test("modal has confirm and close buttons", async ({ page }) => {
    await mockDbActionApi(page, DB_DELETE_API);
    const row = page.getByTestId("table-row").first();
    await row.locator('button[title="Display menu"]').click();
    await page.getByTestId("menu-item-delete").click();
    await expect(page.getByTestId("modal-button-close")).toBeVisible();
    await expect(page.getByTestId("modal-button-confirm")).toBeVisible();
  });

  test("close button closes modal", async ({ page }) => {
    await mockDbActionApi(page, DB_DELETE_API);
    const row = page.getByTestId("table-row").first();
    await row.locator('button[title="Display menu"]').click();
    await page.getByTestId("menu-item-delete").click();
    await expect(page.locator(".modal")).toBeVisible();
    await page.getByTestId("modal-button-close").click();
    await expect(page.locator(".modal")).not.toBeVisible();
  });

  test("close (X) button closes modal", async ({ page }) => {
    await mockDbActionApi(page, DB_DELETE_API);
    const row = page.getByTestId("table-row").first();
    await row.locator('button[title="Display menu"]').click();
    await page.getByTestId("menu-item-delete").click();
    await expect(page.locator(".modal")).toBeVisible();
    await page.getByTestId("close-modal").click();
    await expect(page.locator(".modal")).not.toBeVisible();
  });

  test("confirm button triggers action and closes modal", async ({ page }) => {
    let apiCalled = false;
    await page.route(DB_DELETE_API, async (route) => {
      apiCalled = true;
      route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({}),
      });
    });
    // Re-mock the db list so refresh after action works
    await mockDatabaseListApi(page, []);

    const row = page.getByTestId("table-row").first();
    await row.locator('button[title="Display menu"]').click();
    await page.getByTestId("menu-item-delete").click();
    await page.getByTestId("modal-button-confirm").click();
    await expect(page.locator(".modal")).not.toBeVisible();
    expect(apiCalled).toBe(true);
  });

  test("modal with text input captures value", async ({ page }) => {
    let capturedUrl = "";
    await page.route(DB_COPY_API, async (route) => {
      capturedUrl = route.request().url();
      route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({}),
      });
    });
    await mockDatabaseListApi(page);

    const row = page.getByTestId("table-row").first();
    await row.locator('button[title="Display menu"]').click();
    await page.getByTestId("menu-item-copy").click();

    // Modal should show a text input for the new name
    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();
    const nameInput = modal.locator('input[type="text"]');
    await expect(nameInput).toBeVisible();
    await nameInput.fill("copied_db");
    await page.getByTestId("modal-button-confirm").click();
    await expect(modal).not.toBeVisible();

    expect(capturedUrl).toContain("copied_db");
  });

  test("cancel closes modal without API call", async ({ page }) => {
    let apiCalled = false;
    await page.route(DB_BACKUP_API, async (route) => {
      apiCalled = true;
      route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({}),
      });
    });

    const row = page.getByTestId("table-row").first();
    await row.locator('button[title="Display menu"]').click();
    await page.getByTestId("menu-item-backup").click();
    await expect(page.locator(".modal")).toBeVisible();
    await page.getByTestId("modal-button-close").click();
    await expect(page.locator(".modal")).not.toBeVisible();
    expect(apiCalled).toBe(false);
  });
});
