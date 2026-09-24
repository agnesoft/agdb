import { expect } from "@playwright/test";
import { test } from "./fixtures/agdb.fixtures";
import { testIds, dynamicIds } from "./test-ids";
import { DB_ADD_API, DB_LIST_API } from "./api-paths";
import { mockDatabaseListApi, mockLogin, gotoDbPage } from "./mocks/api-helpers";
import { MOCK_DATABASE_LIST } from "./mocks/db.mock";
import type { ServerDatabase } from "@agnesoft/agdb_api/openapi" with {
  "resolution-mode": "import",
};

test.describe("Database Table", () => {
  test.beforeEach(async ({ ui, page }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await gotoDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();
  });

  test("should display database table with correct data", async ({
    ui,
    page,
  }) => {
    const rows = ui.locator(testIds.TABLE_ROW);
    await expect(rows).toHaveCount(3);

    for (let i = 0; i < 3; i++) {
      const row = rows.nth(i);
      await expect(row).toBeVisible();
      await expect(
        row.getByTestId(dynamicIds.tableCell("db") as string),
      ).toHaveText(["users", "orders", "products"][i] ?? "");
      await expect(
        row.getByTestId(dynamicIds.tableCell("owner") as string),
      ).toHaveText("admin");
      await expect(
        row.getByTestId(dynamicIds.tableCell("db_type") as string),
      ).toHaveText("memory");
      await expect(
        row.getByTestId(dynamicIds.tableCell("role") as string),
      ).toHaveText("admin");
      await expect(
        row.getByTestId(dynamicIds.tableCell("size") as string),
      ).toHaveText("2568");
      await expect(
        row.getByTestId(dynamicIds.tableCell("backup") as string),
      ).toHaveText(
        ["N/A", new Date(1754213481 * 1000).toUTCString(), "N/A"][i] ?? "",
      );
    }
  });

  test("should handle empty database list", async ({ ui, page }) => {
    await mockDatabaseListApi(page, []);
    await ui.click(testIds.REFRESH_BUTTON);
    const rows = ui.locator(testIds.TABLE_ROW);
    await expect(rows).toHaveCount(0);
    await ui
      .element(testIds.EMPTY_TABLE_MESSAGE)
      .hasText("No databases found");
  });

  test("should handle API errors gracefully", async ({ ui, page }) => {
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

  test("should add new database", async ({ ui, page }) => {
    let addApiUrl = "";
    await page.route(DB_ADD_API, async (route) => {
      addApiUrl = route.request().url();
      route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({}),
      });
    });

    const rows = ui.locator(testIds.TABLE_ROW);
    await expect(rows).toHaveCount(3);

    const updatedList: ServerDatabase[] = [
      ...MOCK_DATABASE_LIST,
      {
        db: "new_db",
        owner: "admin",
        db_type: "memory",
        role: "admin",
        size: 2568,
        backup: 0,
        created: 0,
      },
    ];
    await mockDatabaseListApi(page, updatedList);

    await ui.fill(testIds.DB_NAME_INPUT, "new_db");
    await ui.click(testIds.ADD_DB_BUTTON);
    // Verify the add API was called with the right db name
    expect(addApiUrl).toContain("new_db");
    await ui.click(testIds.REFRESH_BUTTON);
    await expect(rows).toHaveCount(4);
    const row = rows.nth(3);
    await expect(row).toBeVisible();
    await expect(
      row.getByTestId(dynamicIds.tableCell("db") as string),
    ).toHaveText("new_db");
  });

  test("should prevent adding database with empty name", async ({ ui }) => {
    const rows = ui.locator(testIds.TABLE_ROW);
    await expect(rows).toHaveCount(3);
    await ui.fill(testIds.DB_NAME_INPUT, "");
    await ui.click(testIds.ADD_DB_BUTTON);
    await expect(rows).toHaveCount(3);
  });
});
