import { expect } from "@playwright/test";
import { test } from "./fixtures/agdb.fixtures";
import { testIds } from "./test-ids";
import {
  mockDatabaseListApi,
  mockLogin,
  mockDbUserListApi,
  mockDbActionApi,
  gotoDbPage,
} from "./mocks/api-helpers";
import { DB_USER_ADD_API, DB_USER_REMOVE_API } from "./api-paths";
import { MOCK_DB_USERS } from "./mocks/user.mock";

test.describe("Database Details — User Management", () => {
  test.beforeEach(async ({ ui, page }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await mockDbUserListApi(page);
    await gotoDbPage(page);
    await ui.element(testIds.DB_TABLE).isVisible();
  });

  const expandFirstRow = async (page: import("@playwright/test").Page) => {
    const row = page.getByTestId("table-row").first();
    const expandButton = row.locator("button.expand-row");
    await expandButton.click();
    const expandedRow = page.locator(".expanded-row");
    await expect(expandedRow).toBeVisible();
    return expandedRow;
  };

  test("expands database row to show details panel", async ({ page }) => {
    const details = await expandFirstRow(page);
    await expect(details).toBeVisible();
    await expect(details.locator(".db-details")).toBeVisible();
  });

  test("displays database name in details header", async ({ page }) => {
    const details = await expandFirstRow(page);
    await expect(details.locator("h2")).toContainText("Database:");
  });

  test("displays list of database users with roles", async ({ page }) => {
    const details = await expandFirstRow(page);
    const userItems = details.locator(".user-item");
    await expect(userItems).toHaveCount(MOCK_DB_USERS.length);

    // Users are grouped by role: Admin group first, then Read Only
    const roleGroups = details.locator(".role-group");
    await expect(roleGroups).toHaveCount(2);

    // Verify admin group
    const adminGroup = roleGroups.first();
    await expect(adminGroup.locator(".role-group-header")).toContainText(
      "Admin",
    );
    await expect(adminGroup.locator(".username")).toHaveText("admin");

    // Verify read group
    const readGroup = roleGroups.nth(1);
    await expect(readGroup.locator(".role-group-header")).toContainText(
      "Read Only",
    );
    await expect(readGroup.locator(".username")).toHaveText("reader");
  });

  test("shows add user button for admin role", async ({ page }) => {
    // Mock databases include role: "admin", so the add button should be visible
    const details = await expandFirstRow(page);
    const addButton = details.locator("button.add-button");
    await expect(addButton).toBeVisible();
  });

  test("add user: opens modal and confirms", async ({ page }) => {
    await mockDbActionApi(page, DB_USER_ADD_API);
    await mockDbUserListApi(page);

    const details = await expandFirstRow(page);
    await details.locator("button.add-button").click();

    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();
    await expect(modal.locator(".modal-header h3")).toContainText("Add user");

    // Fill username input
    const usernameInput = modal.locator('input[type="text"]');
    await usernameInput.fill("newuser");

    // Role select should be present
    const roleSelect = modal.locator("select");
    await expect(roleSelect).toBeVisible();

    await page.getByTestId("modal-button-confirm").click();
  });

  test("remove user: opens confirmation modal", async ({ page }) => {
    await mockDbActionApi(page, DB_USER_REMOVE_API);
    await mockDbUserListApi(page);

    const details = await expandFirstRow(page);
    // Remove button should exist for non-owner users
    const removeButtons = details.locator("button.remove-button");
    // Only the "reader" user should have a remove button (admin is the owner)
    await expect(removeButtons).toHaveCount(1);

    await removeButtons.first().click();

    const modal = page.locator(".modal");
    await expect(modal).toBeVisible();
    await expect(modal.locator(".modal-header h3")).toContainText(
      "Remove user",
    );
    await expect(modal).toContainText("reader");
  });

  test("owner user cannot be removed", async ({ page }) => {
    const details = await expandFirstRow(page);
    // The owner (admin) should not have a remove button
    const adminItem = details.locator(".user-item").first();
    const removeButton = adminItem.locator("button.remove-button");
    await expect(removeButton).toHaveCount(0);
  });
});
