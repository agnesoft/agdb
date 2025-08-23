import { expect, type Page } from "@playwright/test";
import { test } from "../e2e-utils/global.setup";
import { mockLogin } from "../e2e-utils/utils";
import { DB_ADD_API, DB_LIST_API } from "../e2e-utils/apiPaths";
import {
  containsText,
  getLocatorByTestId,
  getSelectorByTestId,
  hasText,
  isVisible,
} from "../e2e-utils/elements";
import { click, fillInput } from "../e2e-utils/interaction";
import type { ServerDatabase } from "@agnesoft/agdb_api/openapi" with { "resolution-mode": "import" };

const mockedDatabaseList: ServerDatabase[] = [
  {
    db: "users",
    owner: "admin",
    db_type: "memory",
    role: "admin",
    size: 2568,
    backup: 0,
  },
  {
    db: "orders",
    owner: "admin",
    db_type: "memory",
    role: "admin",
    size: 2568,
    backup: 1754213481,
  },
  {
    db: "products",
    owner: "admin",
    db_type: "memory",
    role: "admin",
    size: 2568,
    backup: 0,
  },
];

const mockDatabaseListApi = async (
  page: Page,
  databaseList: ServerDatabase[],
) => {
  await page.route(DB_LIST_API, async (route) => {
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(databaseList),
    });
  });
};

test.describe("Database Table E2E Tests", () => {
  test.beforeEach(async ({ page }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page, mockedDatabaseList);
    await page.goto("/studio/db");
    await isVisible(page, "db-table");
  });

  test("should display database table with correct data", async ({ page }) => {
    const rows = getLocatorByTestId(page, "table-row");
    await expect(rows).toHaveCount(3);

    for (let i = 0; i < 3; i++) {
      const row = rows.nth(i);
      await expect(row).toBeVisible();
      await expect(
        row.locator(getSelectorByTestId("table-cell-db")),
      ).toHaveText(["users", "orders", "products"][i]);
      await expect(
        row.locator(getSelectorByTestId("table-cell-owner")),
      ).toHaveText("admin");
      await expect(
        row.locator(getSelectorByTestId("table-cell-db_type")),
      ).toHaveText("memory");
      await expect(
        row.locator(getSelectorByTestId("table-cell-role")),
      ).toHaveText("admin");
      await expect(
        row.locator(getSelectorByTestId("table-cell-size")),
      ).toHaveText("2568");
      await expect(
        row.locator(getSelectorByTestId("table-cell-backup")),
      ).toHaveText(
        ["N/A", new Date(1754213481 * 1000).toUTCString(), "N/A"][i],
      );
    }
  });

  test("should handle empty database list", async ({ page }) => {
    mockDatabaseListApi(page, []);
    await click(page, "refresh-button");
    const rows = getLocatorByTestId(page, "table-row");
    await expect(rows).toHaveCount(0);
    await hasText(page, "empty-table-message", "No databases found");
  });

  test("should handle API errors gracefully", async ({ page }) => {
    await page.route(DB_LIST_API, async (route) => {
      route.fulfill({
        status: 500,
        contentType: "application/json",
        body: JSON.stringify({ error: "Internal Server Error" }),
      });
    });
    await click(page, "refresh-button");
    await containsText(page, "notification-item", "Internal Server Error");
  });

  test("should add new database", async ({ page }) => {
    await page.route(DB_ADD_API, async (route) => {
      const request = route.request();
      const body = await request.postDataJSON();
      expect(body).toEqual({
        db: "new_db",
        db_type: "memory",
        owner: "admin",
        role: "admin",
      });
      route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          db: "new_db",
          owner: "admin",
          db_type: "memory",
          role: "admin",
          size: 2568,
          backup: 0,
        }),
      });
    });

    const rows = getLocatorByTestId(page, "table-row");
    await expect(rows).toHaveCount(3);
    await mockDatabaseListApi(page, [
      ...mockedDatabaseList,
      {
        db: "new_db",
        owner: "admin",
        db_type: "memory",
        role: "admin",
        size: 2568,
        backup: 0,
      },
    ]);
    await fillInput(page, "db-name-input", "new_db");
    await click(page, "add-db-button");
    await expect(rows).toHaveCount(4);
    const row = rows.nth(3);
    await expect(row).toBeVisible();
    await expect(row.locator(getSelectorByTestId("table-cell-db"))).toHaveText(
      "new_db",
    );
  });
  test("should prevent adding database with empty name", async ({ page }) => {
    const rows = getLocatorByTestId(page, "table-row");
    await expect(rows).toHaveCount(3);
    await fillInput(page, "db-name-input", "");
    await click(page, "add-db-button");
    await expect(rows).toHaveCount(3);
  });
});
