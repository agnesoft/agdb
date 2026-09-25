import { expect } from "@playwright/test";
import { test } from "./fixtures/agdb.fixtures";
import { testIds } from "./test-ids";
import {
  mockDatabaseListApi,
  mockLogin,
  mockDbActionApi,
  mockDbAuditApi,
  gotoDbPage,
} from "./mocks/api-helpers";
import {
  DB_BACKUP_API,
  DB_RESTORE_API,
  DB_DELETE_API,
  DB_REMOVE_API,
  DB_OPTIMIZE_API,
  DB_RENAME_API,
  DB_COPY_API,
  DB_CONVERT_API,
  DB_CLEAR_API,
  DB_ROLLBACK_API,
} from "./api-paths";

test.describe("Database Row Actions", () => {
  test.beforeEach(async ({ ui, page }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await gotoDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();
  });

  const openActionMenu = async (page: import("@playwright/test").Page) => {
    const row = page.getByTestId("table-row").first();
    await row.locator('button[title="Display menu"]').click();
  };

  test("opens action menu on row", async ({ page }) => {
    await openActionMenu(page);
    // Menu should be visible with action items
    await expect(page.getByTestId("menu-item-backup")).toBeVisible();
    await expect(page.getByTestId("menu-item-delete")).toBeVisible();
  });

  test("backup: confirm calls API and shows notification", async ({
    ui,
    page,
  }) => {
    await mockDbActionApi(page, DB_BACKUP_API);
    await mockDatabaseListApi(page);
    await openActionMenu(page);
    await page.getByTestId("menu-item-backup").click();

    await expect(page.locator(".modal")).toBeVisible();
    await page.getByTestId("modal-button-confirm").click();
    await expect(page.locator(".modal")).not.toBeVisible();
    await ui.element(testIds.NOTIFICATION_ITEM).containsText("Backup created");
  });

  test("backup: cancel closes modal without API call", async ({ page }) => {
    let apiCalled = false;
    await page.route(DB_BACKUP_API, async (route) => {
      apiCalled = true;
      route.fulfill({ status: 200, body: "{}" });
    });

    await openActionMenu(page);
    await page.getByTestId("menu-item-backup").click();
    await expect(page.locator(".modal")).toBeVisible();
    await page.getByTestId("modal-button-close").click();
    await expect(page.locator(".modal")).not.toBeVisible();
    expect(apiCalled).toBe(false);
  });

  test("restore: confirm calls API", async ({ ui, page }) => {
    await mockDbActionApi(page, DB_RESTORE_API);
    await mockDatabaseListApi(page);
    await openActionMenu(page);
    await page.getByTestId("menu-item-restore").click();

    await expect(page.locator(".modal")).toBeVisible();
    await page.getByTestId("modal-button-confirm").click();
    await expect(page.locator(".modal")).not.toBeVisible();
    await ui
      .element(testIds.NOTIFICATION_ITEM)
      .containsText("restored successfully");
  });

  test("delete: confirm calls API and shows notification", async ({
    ui,
    page,
  }) => {
    await mockDbActionApi(page, DB_DELETE_API);
    await mockDatabaseListApi(page, []);
    await openActionMenu(page);
    await page.getByTestId("menu-item-delete").click();

    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();
    await expect(modal).toContainText("permanently delete");
    await page.getByTestId("modal-button-confirm").click();
    await expect(modal).not.toBeVisible();
    await ui
      .element(testIds.NOTIFICATION_ITEM)
      .containsText("deleted successfully");
  });

  test("remove: confirm calls API", async ({ ui, page }) => {
    await mockDbActionApi(page, DB_REMOVE_API);
    await mockDatabaseListApi(page);
    await openActionMenu(page);
    await page.getByTestId("menu-item-remove").click();

    await expect(page.locator(".modal")).toBeVisible();
    await page.getByTestId("modal-button-confirm").click();
    await ui
      .element(testIds.NOTIFICATION_ITEM)
      .containsText("removed successfully");
  });

  test("optimize: confirm calls API", async ({ ui, page }) => {
    await mockDbActionApi(page, DB_OPTIMIZE_API);
    await mockDatabaseListApi(page);
    await openActionMenu(page);
    await page.getByTestId("menu-item-optimize").click();

    await expect(page.locator(".modal")).toBeVisible();
    await page.getByTestId("modal-button-confirm").click();
    await ui
      .element(testIds.NOTIFICATION_ITEM)
      .containsText("optimized successfully");
  });

  test("rename: fills name input and confirms", async ({ ui, page }) => {
    let capturedUrl = "";
    await page.route(DB_RENAME_API, async (route) => {
      capturedUrl = route.request().url();
      route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({}),
      });
    });
    await mockDatabaseListApi(page);
    await openActionMenu(page);
    await page.getByTestId("menu-item-rename").click();

    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();
    // Rename modal has a text input pre-filled with current name
    const nameInput = modal.locator('input[type="text"]').first();
    await nameInput.clear();
    await nameInput.fill("renamed_db");
    await page.getByTestId("modal-button-confirm").click();

    expect(capturedUrl).toContain("renamed_db");
    await ui
      .element(testIds.NOTIFICATION_ITEM)
      .containsText("renamed successfully");
  });

  test("copy: fills name input and confirms", async ({ ui, page }) => {
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
    await openActionMenu(page);
    await page.getByTestId("menu-item-copy").click();

    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();
    const nameInput = modal.locator('input[type="text"]').first();
    await nameInput.fill("copied_db");
    await page.getByTestId("modal-button-confirm").click();

    expect(capturedUrl).toContain("copied_db");
    await ui
      .element(testIds.NOTIFICATION_ITEM)
      .containsText("copied successfully");
  });

  test("convert to memory: confirm calls API", async ({ ui, page }) => {
    await mockDbActionApi(page, DB_CONVERT_API);
    await mockDatabaseListApi(page);
    await openActionMenu(page);

    // Hover over "Convert" to open submenu
    await page.getByTestId("menu-item-convert").hover();
    await page.getByTestId("menu-item-memory").click();

    await expect(page.locator(".modal")).toBeVisible();
    await page.getByTestId("modal-button-confirm").click();
    await ui
      .element(testIds.NOTIFICATION_ITEM)
      .containsText("converted to memory");
  });

  test("clear all: confirm calls API", async ({ ui, page }) => {
    await mockDbActionApi(page, DB_CLEAR_API);
    await mockDatabaseListApi(page);
    await openActionMenu(page);

    // Hover over "Clear" to open submenu
    await page.getByTestId("menu-item-clear").hover();
    await page.getByTestId("menu-item-all").click();

    await expect(page.locator(".modal")).toBeVisible();
    await page.getByTestId("modal-button-confirm").click();
    await ui
      .element(testIds.NOTIFICATION_ITEM)
      .containsText("cleared successfully");
  });

  test("audit: shows audit log in modal", async ({ page }) => {
    await mockDbAuditApi(page);
    await openActionMenu(page);
    await page.getByTestId("menu-item-audit").click();

    // Audit opens a modal directly (no confirmation) with audit log content
    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();
    await expect(modal.locator(".modal-header h3")).toContainText("Audit log");
    await expect(modal).toContainText("SELECT *");
    // Audit modal has only a Close button (no Confirm)
    await expect(page.getByTestId("modal-button-close")).toBeVisible();
  });

  test("rollback: confirm calls API", async ({ ui, page }) => {
    await mockDbActionApi(page, DB_ROLLBACK_API);
    await mockDatabaseListApi(page);
    await openActionMenu(page);
    await page.getByTestId("menu-item-rollback").click();

    await expect(page.locator(".modal")).toBeVisible();
    await page.getByTestId("modal-button-confirm").click();
    await ui
      .element(testIds.NOTIFICATION_ITEM)
      .containsText("rolled back successfully");
  });
});
