import { expect } from "@playwright/test";
import { test } from "./fixtures/agdb.fixtures";
import { testIds, dynamicIds } from "./test-ids";
import {
  mockDatabaseListApi,
  mockLogin,
  mockDbUserListApi,
  mockDbExecApi,
  gotoDbPage,
} from "./mocks/api-helpers";

test.describe("Query View", () => {
  test.beforeEach(async ({ ui, page }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await mockDbUserListApi(page);
    await mockDbExecApi(page);
  });

  test("navigates to query view when clicking a database row", async ({
    ui,
    page,
  }) => {
    await gotoDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();

    // Click on the first row (not on action buttons)
    const row = page.getByTestId("table-row").first();
    // Click on the db name cell to navigate
    await row.getByTestId("table-cell-db").click();

    await expect(page).toHaveURL(/.*\/studio\/query\/admin\/users/);
  });

  test("displays database name in heading", async ({ page }) => {
    await page.goto("/studio/query/admin/users");
    await expect(page.locator("h1")).toContainText("admin/users");
    await expect(page.locator("h1")).toContainText("query");
  });

  test("renders query builder tabs", async ({ page }) => {
    await page.goto("/studio/query/admin/users");
    const tabs = page.locator('[role="tab"]');
    await expect(tabs).toHaveCount(3);
    await expect(tabs.nth(0)).toHaveText("exec");
    await expect(tabs.nth(1)).toHaveText("exec_mut");
    await expect(tabs.nth(2)).toHaveText("context");
  });

  test("exec tab is active by default", async ({ page }) => {
    await page.goto("/studio/query/admin/users");
    const execTab = page.locator('[role="tab"]').nth(0);
    await expect(execTab).toHaveAttribute("aria-selected", "true");
  });

  test("switching tabs changes active tab", async ({ page }) => {
    await page.goto("/studio/query/admin/users");
    const tabs = page.locator('[role="tab"]');

    // Click on exec_mut tab
    await tabs.nth(1).click();
    await expect(tabs.nth(1)).toHaveAttribute("aria-selected", "true");
    await expect(tabs.nth(0)).toHaveAttribute("aria-selected", "false");

    // Click on context tab
    await tabs.nth(2).click();
    await expect(tabs.nth(2)).toHaveAttribute("aria-selected", "true");
    await expect(tabs.nth(1)).toHaveAttribute("aria-selected", "false");
  });
});
