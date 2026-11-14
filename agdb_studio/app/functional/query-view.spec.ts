import { expect, type Page } from "@playwright/test";
import { test } from "./fixtures/agdb.fixtures";
import { dynamicIds, testIds } from "./test-ids";
import {
  mockDatabaseListApi,
  mockLogin,
  mockDbUserListApi,
  mockDbExecApi,
  gotoDbPage,
  gotoAndWaitForLoad,
} from "./mocks/api-helpers";
import { DB_EXEC_API, DB_EXEC_MUT_API } from "./api-paths";

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

    // Drop onto the last step, on the wrapped row's last line of steps: it
    // lands after the row. Aim at the step's bottom edge, as the row's
    // buttons may wrap onto a line of their own below it.
    const lastStep = rows(page).nth(1).getByTestId(testIds.QUERY_STEP).last();
    const stepBox = (await lastStep.boundingBox())!;
    await handle(page, 0).dragTo(lastStep, {
      targetPosition: { x: stepBox.width / 2, y: stepBox.height - 2 },
    });

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

const kv = (key: string, value: string) => ({
  key: { String: key },
  value: { String: value },
});

const ALICE = { id: 1, from: null, to: null, values: [kv("name", "alice")] };
const BOB = { id: 2, from: null, to: null, values: [kv("name", "bob")] };
const KNOWS = { id: -3, from: 1, to: 2, values: [kv("since", "2020")] };

/**
 * Answers the run query with `results`, and the panel's lookup of stored
 * values (a select by ids) with the matching elements.
 */
const mockQueries = async (page: Page, results: unknown[]) => {
  await page.route(DB_EXEC_API, (route) => {
    const [query] = route.request().postDataJSON() ?? [];
    const ids: { Id: number }[] | undefined = query?.SelectValues?.ids?.Ids;
    const body = ids
      ? [
          {
            result: ids.length,
            elements: [ALICE, BOB, KNOWS].filter((element) =>
              ids.some((id) => id.Id === element.id),
            ),
          },
        ]
      : results;
    return route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(body),
    });
  });
};

const mockMutations = async (page: Page) => {
  await page.route(DB_EXEC_MUT_API, (route) =>
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify([{ result: 1, elements: [] }]),
    }),
  );
};

const runQuery = (page: Page) =>
  page.getByRole("button", { name: "Run query" }).click();

test.describe("Query results", () => {
  test.beforeEach(async ({ page }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await mockDbUserListApi(page);
  });

  test("shows a count as a single number", async ({ page }) => {
    await mockQueries(page, [{ result: 1234, elements: [] }]);
    await gotoAndWaitForLoad(page, QUERY_PAGE);
    await addSteps(page, 0, ["select", "node_count"]);
    await runQuery(page);

    const scalar = page.getByTestId(testIds.RESULT_SCALAR);
    await expect(scalar).toContainText("Node count");
    await expect(scalar).toContainText("1,234");
    await expect(page.getByTestId(testIds.RESULTS_CHIP)).toHaveCount(1);
  });

  test("shows elements in a table and as JSON", async ({ page }) => {
    await mockQueries(page, [{ result: 3, elements: [ALICE, BOB, KNOWS] }]);
    await gotoAndWaitForLoad(page, QUERY_PAGE);
    await addSteps(page, 0, ["search", "elements"]);
    await runQuery(page);

    // Graph elements open in the graph view.
    await expect(page.getByTestId(testIds.VIEW_GRAPH)).toHaveAttribute(
      "aria-pressed",
      "true",
    );

    await page.getByTestId(testIds.VIEW_TABLE).click();
    const table = page.getByTestId(testIds.RESULTS_TABLE);
    await expect(table.locator("th")).toHaveText([
      "",
      "id",
      "from",
      "to",
      "name",
      "since",
    ]);
    const tableRows = page.getByTestId(testIds.RESULTS_TABLE_ROW);
    await expect(tableRows).toHaveCount(3);
    await expect(tableRows.nth(2)).toContainText("-3");
    await expect(tableRows.nth(2)).toContainText("2020");

    await page.getByTestId(testIds.VIEW_JSON).click();
    await expect(page.getByTestId(testIds.RESULTS_JSON)).toContainText(
      '"alice"',
    );
  });

  test("views and edits an element's values from the table", async ({
    page,
  }) => {
    await mockQueries(page, [{ result: 1, elements: [ALICE] }]);
    await mockMutations(page);
    await gotoAndWaitForLoad(page, QUERY_PAGE);
    await addSteps(page, 0, ["search", "elements"]);
    await runQuery(page);
    await page.getByTestId(testIds.VIEW_TABLE).click();

    await page.getByTestId(testIds.RESULTS_ROW_MENU).click();
    await page.getByTestId(dynamicIds.menuItem("view")).click();
    const panel = page.getByTestId(testIds.ELEMENT_DETAILS_PANEL);
    await expect(panel.getByTestId(testIds.PANEL_VALUE_ROW)).toHaveText(
      /name.*alice/,
    );

    await panel.getByRole("button", { name: "Edit values" }).click();
    await panel.getByTestId(testIds.PANEL_VALUE_INPUT).fill("carol");
    const request = page.waitForRequest(DB_EXEC_MUT_API);
    await panel.getByTestId(testIds.PANEL_SAVE).click();
    expect((await request).postDataJSON()).toEqual([
      {
        InsertValues: {
          ids: { Ids: [{ Id: 1 }] },
          values: { Multi: [[kv("name", "carol")]] },
        },
      },
    ]);
    await expect(page.getByText("Values saved")).toBeVisible();
  });

  test("deletes an element after confirmation", async ({ page }) => {
    await mockQueries(page, [{ result: 1, elements: [ALICE] }]);
    await mockMutations(page);
    await gotoAndWaitForLoad(page, QUERY_PAGE);
    await addSteps(page, 0, ["search", "elements"]);
    await runQuery(page);
    await page.getByTestId(testIds.VIEW_TABLE).click();

    await page.getByTestId(testIds.RESULTS_ROW_MENU).click();
    await page.getByTestId(dynamicIds.menuItem("delete")).click();
    const request = page.waitForRequest(DB_EXEC_MUT_API);
    await page.getByTestId(dynamicIds.modalButton("confirm")).click();
    expect((await request).postDataJSON()).toEqual([
      { Remove: { Ids: [{ Id: 1 }] } },
    ]);
    await expect(page.getByText("Element deleted")).toBeVisible();
  });
});

test.describe("Query copy and paste", () => {
  test.beforeEach(async ({ page, context }) => {
    await context.grantPermissions(["clipboard-read", "clipboard-write"]);
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await mockDbUserListApi(page);
    await mockDbExecApi(page);
    await gotoAndWaitForLoad(page, QUERY_PAGE);
    await expect(rows(page)).toHaveCount(1);
  });

  const clipboard = (page: Page) =>
    page.evaluate(() => navigator.clipboard.readText());

  const setClipboard = (page: Page, text: string) =>
    page.evaluate((value) => navigator.clipboard.writeText(value), text);

  test("copies a query as Rust code and pastes it into another row", async ({
    page,
  }) => {
    await addSteps(page, 0, ["select", "node_count"]);
    await rows(page)
      .nth(0)
      .getByRole("button", { name: "Copy this query as Rust code" })
      .click();
    await expect(page.getByText("Query copied")).toBeVisible();
    expect(await clipboard(page)).toBe(
      "QueryBuilder::select().node_count().query()",
    );

    await addQueryButton(page).click();
    await rows(page)
      .nth(1)
      .getByRole("button", { name: "Paste a copied query here" })
      .click();
    expect(await rowQueries(page)).toEqual([
      "select.node_count",
      "select.node_count",
    ]);
  });

  test("pastes Rust code into the step input", async ({ page }) => {
    await setClipboard(
      page,
      'let r = db.exec(&QueryBuilder::search().from("root").query())?;',
    );
    await stepInput(page, 0).click();
    await page.keyboard.press("ControlOrMeta+V");
    // The step shows its argument, e.g. "search.from.(s, root)".
    await expect
      .poll(() => rowQueries(page))
      .toEqual([expect.stringMatching(/^search\.from\.\(.*root\)$/)]);
  });

  test("explains why a paste failed", async ({ page }) => {
    await setClipboard(page, "QueryBuilder::select().fly().query()");
    await rows(page)
      .nth(0)
      .getByRole("button", { name: "Paste a copied query here" })
      .click();
    await expect(page.getByText("Could not paste the query")).toBeVisible();
    await expect(page.getByText("fly() cannot follow select()")).toBeVisible();
    expect(await rowQueries(page)).toEqual([""]);
  });
});
