import { expect, type Page, type Route } from "@playwright/test";
import { test } from "./fixtures/agdb.fixtures";
import {
  gotoAndWaitForLoad,
  mockDatabaseListApi,
  mockDbExecApi,
  mockLogin,
} from "./mocks/api-helpers";
import { DASHBOARD_API, DB_EXEC_API } from "./api-paths";
import { testIds } from "./test-ids";

type StoredDashboard = { name: string; config: string; updated: number };

/** In-memory stand-in for the dashboard endpoints, so a reload sees saved data. */
async function mockDashboardApi(page: Page) {
  const store = new Map<string, StoredDashboard>();
  const json = (route: Route, body: unknown, status = 200) =>
    route.fulfill({
      status,
      contentType: "application/json",
      body: JSON.stringify(body),
    });

  await page.route(DASHBOARD_API, async (route) => {
    const request = route.request();
    const url = new URL(request.url());
    const parts = url.pathname.split("/dashboard/")[1]?.split("/") ?? [];
    const name = decodeURIComponent(parts[0] ?? "");
    const action = parts[1];

    if (name === "list") {
      return json(
        route,
        [...store.values()].map(({ name, updated }) => ({ name, updated })),
      );
    }
    if (!action && request.method() === "GET") {
      const dashboard = store.get(name);
      return dashboard ? json(route, dashboard) : json(route, "", 404);
    }
    if (action === "add" || action === "save") {
      if (action === "add" && store.has(name)) return json(route, "", 409);
      const { config } = request.postDataJSON() as { config: string };
      store.set(name, { name, config, updated: 1_700_000_000 });
      return json(route, "", 201);
    }
    if (action === "rename") {
      const newName = url.searchParams.get("new_name") ?? "";
      const dashboard = store.get(name);
      if (!dashboard) return json(route, "", 404);
      if (store.has(newName)) return json(route, "", 409);
      store.delete(name);
      store.set(newName, { ...dashboard, name: newName });
      return json(route, "", 201);
    }
    if (action === "remove") {
      store.delete(name);
      return route.fulfill({ status: 204 });
    }
    return json(route, "", 400);
  });
  return store;
}

const kv = (key: string, value: string) => ({
  key: { String: key },
  value: { String: value },
});
const ELEMENTS = [
  { id: 1, from: null, to: null, values: [kv("name", "alice")] },
  { id: 2, from: null, to: null, values: [kv("name", "bob")] },
];

/** Answers every read query with two nodes. */
const mockElements = (page: Page) =>
  page.route(DB_EXEC_API, (route) =>
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify([{ result: 2, elements: ELEMENTS }]),
    }),
  );

/** Types each step into the first query row and confirms it with Enter. */
const addSteps = async (page: Page, steps: string[]) => {
  const input = page
    .getByTestId(testIds.QUERY_ROW)
    .first()
    .getByRole("textbox", { name: "Query input" });
  for (const step of steps) {
    await input.click();
    await page.keyboard.type(step);
    await page.keyboard.press("Enter");
  }
  await page.keyboard.press("Escape");
};

test.describe("Dashboards", () => {
  test.beforeEach(async ({ page }) => {
    await mockLogin(page);
    await mockDatabaseListApi(page);
    await mockDbExecApi(page, [{ result: 42, elements: [] }]);
  });

  test("creates a dashboard, builds a widget in the query editor, and keeps each change", async ({
    page,
  }) => {
    const store = await mockDashboardApi(page);
    await mockElements(page);

    await gotoAndWaitForLoad(page, "/studio/dashboards");
    await expect(page.getByTestId("dashboards-empty")).toBeVisible();

    // Create
    await page.getByTestId("dashboard-create").click();
    const modal = page.locator(".modal");
    await modal.getByLabel(/^Name/).fill("ops");
    await page.getByTestId("modal-button-confirm").click();
    await expect(page).toHaveURL(/\/studio\/dashboards\/ops$/);
    expect(store.has("ops")).toBe(true);

    // Add a widget: the editor is the query view with exec and context tabs.
    await page.getByTestId("dashboard-add-widget").click();
    const editor = page.getByTestId("widget-editor");
    await expect(editor).toBeVisible();
    await expect(editor.locator('[role="tab"]')).toHaveText([
      "exec",
      "context",
    ]);
    await expect(editor.getByTestId("widget-editor-done")).toBeDisabled();
    await editor.getByTestId("widget-title").fill("People");
    await expect(editor.getByTestId("widget-database")).toHaveValue(
      "admin/users",
    );
    await addSteps(page, ["search", "elements"]);
    // The preview runs by itself once the query is complete.
    await expect(editor.getByText("result: 2")).toBeVisible();
    await editor.getByTestId(testIds.VIEW_TABLE).click();
    await expect(editor.getByTestId(testIds.RESULTS_TABLE_ROW)).toHaveCount(2);
    await editor.getByTestId("widget-editor-done").click();
    await expect(editor).toBeHidden();

    // Size is set from the widget's menu, not in the editor; it saves at once.
    await page.getByTestId("widget-menu").click();
    await page.getByTestId("widget-size-4x2").hover();
    await page.getByTestId("widget-size-4x2").click();
    await page.keyboard.press("Escape");

    // Rows and the chosen view are the widget; size is the layout.
    await expect
      .poll(() => JSON.parse(store.get("ops")!.config).layout)
      .toEqual([{ id: expect.any(String), cols: 4, rows: 2 }]);
    const saved = JSON.parse(store.get("ops")!.config);
    const [widget] = Object.values(saved.widgets);
    expect(widget).toMatchObject({
      title: "People",
      owner: "admin",
      db: "users",
      view: "table",
      rows: { exec: [{ query: "search().elements()" }], context: [] },
    });
    await expect(page.getByTestId("dashboard-unsaved")).toBeHidden();

    // Switching the view in the widget is not saved until asked to.
    const viewChanged = page.getByTestId("widget-view-changed");
    await expect(viewChanged).toBeHidden();
    await page
      .getByTestId("dashboard-grid")
      .getByTestId(testIds.VIEW_JSON)
      .click();
    await expect(viewChanged).toBeVisible();
    await page.getByTestId("widget-view-reset").click();
    await expect(viewChanged).toBeHidden();

    // Reload: the widget comes back from the server and runs its query.
    await page.reload();
    const grid = page.getByTestId("dashboard-grid");
    await expect(grid).toContainText("People");
    await expect(grid).toContainText("admin/users");
    await expect(grid.getByTestId(testIds.RESULTS_TABLE_ROW)).toHaveCount(2);
    await expect(grid.getByTestId(testIds.RESULTS_TABLE)).toContainText(
      "alice",
    );
  });

  test("pins the query page's query with its view to a new dashboard", async ({
    page,
  }) => {
    const store = await mockDashboardApi(page);
    await mockElements(page);
    await gotoAndWaitForLoad(page, "/studio/query/admin/users");
    const pin = page.getByTestId("pin-to-dashboard");
    await expect(pin).toBeDisabled();

    await addSteps(page, ["search", "elements"]);
    await page.getByRole("button", { name: "Run query" }).click();
    await page.getByTestId(testIds.VIEW_TABLE).click();
    await pin.click();
    const modal = page.locator(".modal");
    await modal.getByLabel(/^New dashboard name/).fill("people");
    await modal.getByLabel(/^Widget title/).fill("Everyone");
    await page.getByTestId("modal-button-confirm").click();
    await expect(modal).toBeHidden();

    const [widget] = Object.values(
      JSON.parse(store.get("people")!.config).widgets,
    );
    expect(widget).toMatchObject({
      title: "Everyone",
      view: "table",
      rows: { exec: [{ query: "search().elements()" }] },
    });
  });

  test("renames and deletes a dashboard from the list", async ({ page }) => {
    const store = await mockDashboardApi(page);
    store.set("ops", { name: "ops", config: "{}", updated: 1_700_000_000 });

    await gotoAndWaitForLoad(page, "/studio/dashboards");
    await expect(page.getByText("ops", { exact: true })).toBeVisible();

    const modal = page.locator(".modal");
    await page.getByTestId("dashboard-rename").click();
    await modal.getByLabel(/^Name/).fill("sales");
    await page.getByTestId("modal-button-confirm").click();
    await expect(page.getByText("sales", { exact: true })).toBeVisible();
    expect([...store.keys()]).toEqual(["sales"]);

    await page.getByTestId("dashboard-remove").click();
    await page.getByTestId("modal-button-confirm").click();
    await expect(page.getByTestId("dashboards-empty")).toBeVisible();
    expect(store.size).toBe(0);
  });
});
