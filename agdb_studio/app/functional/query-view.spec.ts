import { expect, type Page } from "@playwright/test";
import { test } from "./fixtures/agdb.fixtures";
import { testIds } from "./test-ids";
import {
  mockDatabaseListApi,
  mockLogin,
  mockDbUserListApi,
  mockDbExecApi,
  gotoDbPage,
  gotoAndWaitForLoad,
} from "./mocks/api-helpers";
import { DB_EXEC_API } from "./api-paths";

test.describe("Query View", () => {
  test.beforeEach(async ({ page }) => {
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
    await gotoAndWaitForLoad(page, "/studio/query/admin/users");
    await expect(page.locator("h1")).toContainText("admin/users");
    await expect(page.locator("h1")).toContainText("query");
  });

  test("renders query builder tabs", async ({ page }) => {
    await gotoAndWaitForLoad(page, "/studio/query/admin/users");
    const tabs = page.locator('[role="tab"]');
    await expect(tabs).toHaveCount(3);
    await expect(tabs.nth(0)).toHaveText("exec");
    await expect(tabs.nth(1)).toHaveText("exec_mut");
    await expect(tabs.nth(2)).toHaveText("context");
  });

  test("exec tab is active by default", async ({ page }) => {
    await gotoAndWaitForLoad(page, "/studio/query/admin/users");
    const execTab = page.locator('[role="tab"]').nth(0);
    await expect(execTab).toHaveAttribute("aria-selected", "true");
  });

  test("switching tabs changes active tab", async ({ page }) => {
    await gotoAndWaitForLoad(page, "/studio/query/admin/users");
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

const QUERY_PAGE = "/studio/query/admin/users";

const rows = (page: Page) => page.getByTestId(testIds.QUERY_ROW);

const stepInput = (page: Page, row: number) =>
  rows(page).nth(row).getByRole("textbox", { name: "Query input" });

/** Types each step into the row's input and confirms it with Enter. */
const addSteps = async (page: Page, row: number, steps: string[]) => {
  for (const step of steps) {
    await stepInput(page, row).click();
    await page.keyboard.type(step);
    await page.keyboard.press("Enter");
  }
  // Close the hints so they don't cover other rows.
  await page.keyboard.press("Escape");
};

/** The steps of every row, e.g. ["select.node_count", ""]. */
const rowQueries = (page: Page) =>
  rows(page).evaluateAll((elements) =>
    elements.map((row) =>
      [...row.querySelectorAll('[data-testid="query-step"] .label > span')]
        .map((label) => label.textContent?.trim())
        .join("."),
    ),
  );

const addQueryButton = (page: Page) =>
  page.getByRole("button", {
    name: "Add another query to run in the same transaction",
  });

const handle = (page: Page, row: number) =>
  rows(page)
    .nth(row)
    .getByRole("button", { name: "Reorder query, use up and down arrow keys" });

test.describe("Query builder", () => {
  test.beforeEach(async ({ page }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await mockDbUserListApi(page);
    await mockDbExecApi(page);
    await gotoAndWaitForLoad(page, QUERY_PAGE);
    await expect(rows(page)).toHaveCount(1);
  });

  test("a single query has no row numbers or row controls", async ({
    page,
  }) => {
    await expect(page.getByTestId(testIds.QUERY_ROW_INDEX)).toHaveCount(0);
    await expect(handle(page, 0)).toHaveCount(0);
    await expect(
      page.getByRole("button", { name: "Remove this query" }),
    ).toHaveCount(0);
    await expect(
      page.getByRole("button", { name: /Hide queries/ }),
    ).toHaveCount(0);
    await expect(addQueryButton(page)).toBeVisible();
  });

  test("adds and removes queries inline", async ({ page }) => {
    await addQueryButton(page).click();
    await addQueryButton(page).click();
    await expect(rows(page)).toHaveCount(3);
    // The add button only sits on the last row.
    await expect(addQueryButton(page)).toHaveCount(1);
    await expect(rows(page).nth(2).locator(addQueryButton(page))).toBeVisible();

    await rows(page)
      .nth(1)
      .getByRole("button", { name: "Remove this query" })
      .click();
    await expect(rows(page)).toHaveCount(2);
  });

  test("the clear button follows the query and clears only that query", async ({
    page,
  }) => {
    await addSteps(page, 0, ["select", "node_count"]);
    await addQueryButton(page).click();
    await addSteps(page, 1, ["select", "indexes"]);

    const clear = rows(page)
      .nth(0)
      .getByRole("button", { name: "Clear all steps of this query" });
    const lastStep = await rows(page)
      .nth(0)
      .getByTestId(testIds.QUERY_STEP)
      .last()
      .boundingBox();
    const clearBox = await clear.boundingBox();
    // Right after the last step, not pushed to the far end of the row.
    expect(clearBox!.x - (lastStep!.x + lastStep!.width)).toBeLessThan(40);

    await clear.click();
    expect(await rowQueries(page)).toEqual(["", "select.indexes"]);
  });

  test("reorders queries with the keyboard", async ({ page }) => {
    await addSteps(page, 0, ["select", "node_count"]);
    await addQueryButton(page).click();
    await addSteps(page, 1, ["select", "indexes"]);

    await handle(page, 1).focus();
    await page.keyboard.press("ArrowUp");

    expect(await rowQueries(page)).toEqual([
      "select.indexes",
      "select.node_count",
    ]);
    // Focus stays on the moved row's handle so it can keep moving.
    await expect(handle(page, 0)).toBeFocused();
  });

  test("reorders queries by drag and drop, also onto a wrapped row", async ({
    page,
  }) => {
    await addSteps(page, 0, ["select", "node_count"]);
    await addQueryButton(page).click();
    await addSteps(page, 1, ["search", "elements", ...Array(25).fill("where")]);
    const wrappedRow = await rows(page).nth(1).boundingBox();
    const firstRow = await rows(page).nth(0).boundingBox();
    expect(wrappedRow!.height).toBeGreaterThan(firstRow!.height * 1.5);

    // Drop onto the last step, on the wrapped row's second line: it lands after the row.
    await handle(page, 0).dragTo(
      rows(page).nth(1).getByTestId(testIds.QUERY_STEP).last(),
    );

    const queries = await rowQueries(page);
    expect(queries[1]).toBe("select.node_count");
    expect(queries[0]?.startsWith("search.elements.where")).toBe(true);
  });

  test("numbers only non-empty queries and sends only those", async ({
    page,
  }) => {
    await addSteps(page, 0, ["select", "node_count"]);
    await addQueryButton(page).click();
    await addQueryButton(page).click();
    await addSteps(page, 2, ["select", "indexes"]);

    await expect(page.getByTestId(testIds.QUERY_ROW_INDEX)).toHaveText([
      "0",
      "",
      "1",
    ]);

    const request = page.waitForRequest(DB_EXEC_API);
    await page.getByRole("button", { name: "Run query" }).click();
    expect((await request).postDataJSON()).toEqual([
      { SelectNodeCount: {} },
      { SelectIndexes: {} },
    ]);
  });

  test("hides and shows queries per tab", async ({ page }) => {
    await addSteps(page, 0, ["select", "node_count"]);
    await addQueryButton(page).click();
    const summary = page.getByTestId(testIds.QUERY_HIDDEN_SUMMARY);

    await page.getByRole("button", { name: /Hide queries/ }).click();
    await expect(rows(page)).toHaveCount(0);
    await expect(summary).toContainText("1 query");
    await expect(summary).toContainText("select.node_count");

    // exec_mut has its own single row and is not affected.
    await page.getByRole("tab", { name: "exec_mut" }).click();
    await expect(rows(page)).toHaveCount(1);
    await page.getByRole("tab", { name: "exec", exact: true }).click();
    await expect(summary).toBeVisible();

    await summary.click();
    await expect(rows(page)).toHaveCount(2);
  });

  test("the run button keeps its height and the graph stays on screen as queries are added", async ({
    page,
  }) => {
    const runButton = page.getByRole("button", { name: "Run query" });
    const graph = page.getByTestId(testIds.QUERY_GRAPH);
    const runHeight = (await runButton.boundingBox())!.height;
    const graphHeight = (await graph.boundingBox())!.height;

    for (let i = 0; i < 4; i++) await addQueryButton(page).click();
    await expect(rows(page)).toHaveCount(5);

    expect((await runButton.boundingBox())!.height).toBe(runHeight);
    const viewport = page.viewportSize()!;
    // The graph shrinks to make room instead of being pushed off screen.
    await expect
      .poll(async () => (await graph.boundingBox())!.height)
      .toBeLessThan(graphHeight);
    expect(
      (await graph.boundingBox())!.y + (await graph.boundingBox())!.height,
    ).toBeLessThanOrEqual(viewport.height);
  });
});
