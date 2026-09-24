import { expect } from "@playwright/test";
import { test } from "./fixtures/agdb.fixtures";
import { testIds } from "./test-ids";
import {
  mockDatabaseListApi,
  mockLogin,
  mockDbActionApi,
  gotoDbPage,
} from "./mocks/api-helpers";
import { DB_BACKUP_API, DB_LIST_API } from "./api-paths";

test.describe("Notification System", () => {
  test.beforeEach(async ({ ui, page }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await gotoDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();
  });

  test("notifications appear on API error", async ({ ui, page }) => {
    await page.route(DB_LIST_API, async (route) => {
      route.fulfill({
        status: 500,
        contentType: "application/json",
        body: JSON.stringify({ error: "Internal Server Error" }),
      });
    });
    await ui.click(testIds.REFRESH_BUTTON);
    await ui
      .element(testIds.NOTIFICATION_ITEM)
      .containsText("Internal Server Error");
  });

  test("notification icon appears after notification", async ({ ui, page }) => {
    await page.route(DB_LIST_API, async (route) => {
      route.fulfill({
        status: 500,
        contentType: "application/json",
        body: JSON.stringify({ error: "Server Error" }),
      });
    });
    await ui.click(testIds.REFRESH_BUTTON);
    // After an error, the notification wrapper should be visible
    await expect(page.locator(".notification-wrapper")).toBeVisible();
  });

  test("clicking notification icon opens viewer", async ({ ui, page }) => {
    // Trigger a notification via API error
    await page.route(DB_LIST_API, async (route) => {
      route.fulfill({
        status: 500,
        contentType: "application/json",
        body: JSON.stringify({ error: "Test Error" }),
      });
    });
    await ui.click(testIds.REFRESH_BUTTON);
    await expect(page.locator(".notification-wrapper")).toBeVisible();

    // Click notification button to open the viewer
    await page.locator(".notification-button").click();
    await expect(page.locator(".notification-viewer")).toBeVisible();
    await expect(page.locator(".notification-header h3")).toContainText(
      "Notifications",
    );
  });

  test("notification items display in viewer", async ({ ui, page }) => {
    await page.route(DB_LIST_API, async (route) => {
      route.fulfill({
        status: 500,
        contentType: "application/json",
        body: JSON.stringify({ error: "Internal Server Error" }),
      });
    });
    await ui.click(testIds.REFRESH_BUTTON);
    await page.locator(".notification-button").click();
    await expect(page.locator(".notification-viewer")).toBeVisible();
    const items = page.getByTestId("notification-item");
    // Verify at least one notification item is present with the error text
    const count = await items.count();
    expect(count).toBeGreaterThanOrEqual(1);
    await expect(
      items.filter({ hasText: "Internal Server Error" }),
    ).toHaveCount(1);
  });

  test("notifications appear on successful action", async ({ ui, page }) => {
    await mockDbActionApi(page, DB_BACKUP_API);
    await mockDatabaseListApi(page);

    const row = page.getByTestId("table-row").first();
    await row.locator('button[title="Display menu"]').click();
    await page.getByTestId("menu-item-backup").click();
    await page.getByTestId("modal-button-confirm").click();

    // Success notification should appear
    await ui.element(testIds.NOTIFICATION_ITEM).containsText("Backup created");
  });

  test("multiple notifications stack", async ({ ui, page }) => {
    // Trigger first error
    await page.route(DB_LIST_API, async (route) => {
      route.fulfill({
        status: 500,
        contentType: "application/json",
        body: JSON.stringify({ error: "First Error" }),
      });
    });
    await ui.click(testIds.REFRESH_BUTTON);
    await ui.element(testIds.NOTIFICATION_ITEM).isVisible();

    // Trigger second error
    await ui.click(testIds.REFRESH_BUTTON);

    // Open viewer to see all notifications
    await page.locator(".notification-button").click();
    await expect(page.locator(".notification-viewer")).toBeVisible();
    const items = page.getByTestId("notification-item");
    const count = await items.count();
    expect(count).toBeGreaterThanOrEqual(2);
  });
});
