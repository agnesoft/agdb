import { expect, type Page, type Route } from "@playwright/test";
import { test } from "./fixtures/agdb.fixtures";
import {
  gotoAndWaitForLoad,
  mockDatabaseListApi,
  mockDbExecApi,
  mockAdminUserStatus,
  mockLogin,
} from "./mocks/api-helpers";
import {
  ADMIN_DASHBOARD_API,
  ADMIN_DB_EXEC_API,
  DASHBOARD_API,
  DB_EXEC_API,
  HOME_DASHBOARD_API,
} from "./api-paths";
import { testIds } from "./test-ids";

type Role = "admin" | "write" | "read";
type StoredDashboard = {
  owner: string;
  name: string;
  config: string;
  updated: number;
  /** Users the dashboard is shared with; the owner is an admin. */
  users: Map<string, Role>;
  /** Every user can view it (read only without a role). */
  public: boolean;
};

/** The user of the mocked login. */
const ME = "testuser";

const keyOf = (owner: string, name: string) => `${owner}/${name}`;

/**
 * In-memory stand-in for the dashboard and home dashboard endpoints, so a
 * reload sees saved data. Keys are "owner/name".
 */
async function mockDashboardApi(page: Page) {
  const store = new Map<string, StoredDashboard>();
  const home = { key: undefined as string | undefined };
  const json = (route: Route, body: unknown, status = 200) =>
    route.fulfill({
      status,
      contentType: "application/json",
      body: JSON.stringify(body),
    });
  const roleOf = (dashboard: StoredDashboard): Role | undefined =>
    dashboard.users.get(ME) ?? (dashboard.public ? "read" : undefined);
  const memberOf = (dashboard: StoredDashboard) => dashboard.users.has(ME);
  /** Requests refused as the server would; Studio should not send them. */
  const forbidden: string[] = [];

  await page.route(DASHBOARD_API, async (route) => {
    const request = route.request();
    const url = new URL(request.url());
    const parts = (url.pathname.split("/dashboard/")[1] ?? "")
      .split("/")
      .map(decodeURIComponent);

    if (parts[0] === "list") {
      return json(
        route,
        [...store.values()]
          .filter((d) => roleOf(d))
          .map((d) => ({
            owner: d.owner,
            name: d.name,
            updated: d.updated,
            role: roleOf(d),
            public: d.public,
            member: memberOf(d),
          })),
      );
    }
    const [owner = "", name = "", action, username, userAction] = parts;
    const forbid = () => {
      forbidden.push(`${request.method()} ${url.pathname}`);
      return json(route, "", 403);
    };
    const key = keyOf(owner, name);
    const dashboard = store.get(key);

    if (action === "add") {
      if (dashboard) return json(route, "", 409);
      const { config } = request.postDataJSON() as { config: string };
      store.set(key, {
        owner,
        name,
        config,
        updated: 1_700_000_000,
        users: new Map([[owner, "admin"]]),
        public: false,
      });
      return json(route, "", 201);
    }
    if (!dashboard || !roleOf(dashboard)) return json(route, "", 404);
    if (!action && request.method() === "GET") {
      const { users: _, ...rest } = dashboard;
      return json(route, { ...rest, role: roleOf(dashboard) });
    }
    if (action === "public") {
      if (owner !== ME) return forbid();
      dashboard.public = url.searchParams.get("public") === "true";
      return json(route, "", 201);
    }
    if (action === "save") {
      // As the server: read-only (and public-only) viewers cannot save.
      if (roleOf(dashboard) === "read") return forbid();
      const { config } = request.postDataJSON() as { config: string };
      dashboard.config = config;
      return json(route, "", 201);
    }
    if (action === "rename") {
      if (owner !== ME) return forbid();
      const newName = url.searchParams.get("new_name") ?? "";
      const newKey = keyOf(owner, newName);
      if (store.has(newKey)) return json(route, "", 409);
      store.delete(key);
      store.set(newKey, { ...dashboard, name: newName });
      if (home.key === key) home.key = newKey;
      return json(route, "", 201);
    }
    if (action === "remove") {
      if (owner !== ME) return forbid();
      store.delete(key);
      if (home.key === key) home.key = undefined;
      return route.fulfill({ status: 204 });
    }
    if (action === "user" && username === "list") {
      if (!memberOf(dashboard)) return forbid();
      return json(
        route,
        [...dashboard.users].map(([username, role]) => ({ username, role })),
      );
    }
    if (action === "user" && userAction === "add") {
      dashboard.users.set(username!, url.searchParams.get("db_role") as Role);
      return json(route, "", 201);
    }
    if (action === "user" && userAction === "remove") {
      dashboard.users.delete(username!);
      // A public dashboard stays readable, and so stays the home.
      if (username === ME && home.key === key && !dashboard.public)
        home.key = undefined;
      return route.fulfill({ status: 204 });
    }
    return json(route, "", 400);
  });

  await page.route(HOME_DASHBOARD_API, async (route) => {
    const request = route.request();
    if (request.method() === "GET") {
      const dashboard = home.key ? store.get(home.key) : undefined;
      return json(
        route,
        dashboard ? { owner: dashboard.owner, name: dashboard.name } : null,
      );
    }
    if (request.method() === "PUT") {
      const url = new URL(request.url());
      home.key = keyOf(
        url.searchParams.get("owner") ?? "",
        url.searchParams.get("name") ?? "",
      );
      return json(route, "", 201);
    }
    home.key = undefined;
    return route.fulfill({ status: 204 });
  });

  const add = (
    owner: string,
    name: string,
    users: [string, Role][] = [],
    isPublic = false,
  ) =>
    store.set(keyOf(owner, name), {
      owner,
      name,
      config: "{}",
      updated: 1_700_000_000,
      users: new Map([[owner, "admin"], ...users]),
      public: isPublic,
    });

  return { store, home, add, forbidden };
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
    const { store } = await mockDashboardApi(page);
    await mockElements(page);

    await gotoAndWaitForLoad(page, "/studio/dashboards");
    await expect(page.getByTestId("dashboards-empty")).toBeVisible();

    // Create
    await page.getByTestId("dashboard-create").click();
    const modal = page.locator(".modal");
    await modal.getByLabel(/^Name/).fill("ops");
    await page.getByTestId("modal-button-confirm").click();
    await expect(page).toHaveURL(/\/studio\/dashboards\/testuser\/ops$/);
    expect(store.has("testuser/ops")).toBe(true);

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
      .poll(() => JSON.parse(store.get("testuser/ops")!.config).layout)
      .toEqual([{ id: expect.any(String), cols: 4, rows: 2 }]);
    const saved = JSON.parse(store.get("testuser/ops")!.config);
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

  test("builds a chart widget and changes the chart on the dashboard", async ({
    page,
  }) => {
    const { store, add } = await mockDashboardApi(page);
    add(ME, "ops");
    const team = (id: number, name: string) => ({
      id,
      from: null,
      to: null,
      values: [kv("team", name)],
    });
    await page.route(DB_EXEC_API, (route) =>
      route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify([
          {
            result: 3,
            elements: [team(1, "red"), team(2, "red"), team(3, "blue")],
          },
        ]),
      }),
    );
    await gotoAndWaitForLoad(page, "/studio/dashboards/testuser/ops");

    // Picking "Chart" shows a chart that fits the results at once.
    await page.getByTestId("dashboard-add-widget").click();
    const editor = page.getByTestId("widget-editor");
    await editor.getByTestId("widget-title").fill("Teams");
    await addSteps(page, ["search", "elements"]);
    await expect(editor.getByText("result: 3")).toBeVisible();
    await editor.getByTestId(testIds.VIEW_CHART).click();
    await expect(editor.getByTestId("chart-title")).toHaveText("Count by team");
    await expect(editor.getByTestId("chart-bar-0")).toHaveAttribute(
      "aria-label",
      "red: 2",
    );
    await editor.getByTestId("chart-kind-donut").click();
    await expect(editor.getByTestId("chart-donut")).toBeVisible();
    await editor.getByTestId("widget-editor-done").click();
    await expect(editor).toBeHidden();

    const savedWidget = () =>
      Object.values(JSON.parse(store.get("testuser/ops")!.config).widgets)[0];
    await expect.poll(savedWidget).toMatchObject({
      view: "chart",
      chart: {
        kind: "donut",
        groupBy: { by: "property", key: "team" },
        measure: { aggregate: "count" },
      },
    });
    expect(JSON.parse(store.get("testuser/ops")!.config).version).toBe(3);

    // On the dashboard the settings sit behind a button; changes wait for Save.
    const grid = page.getByTestId("dashboard-grid");
    await expect(grid.getByTestId("chart-donut")).toBeVisible();
    await grid.getByTestId("chart-settings-toggle").click();
    await grid.getByTestId("chart-kind-bar").click();
    await expect(grid.getByTestId("chart-bar")).toBeVisible();
    await expect(page.getByTestId("widget-view-changed")).toBeVisible();
    await page.getByTestId("widget-view-save").click();
    await expect.poll(savedWidget).toMatchObject({ chart: { kind: "bar" } });

    // Reload: the chart comes back as saved.
    await page.reload();
    await expect(
      page.getByTestId("dashboard-grid").getByTestId("chart-bar-1"),
    ).toHaveAttribute("aria-label", "blue: 1");
  });

  test("pins the query page's query with its view to a new dashboard", async ({
    page,
  }) => {
    const { store } = await mockDashboardApi(page);
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
      JSON.parse(store.get("testuser/people")!.config).widgets,
    );
    expect(widget).toMatchObject({
      title: "Everyone",
      view: "table",
      rows: { exec: [{ query: "search().elements()" }] },
    });
  });

  test("renames and deletes a dashboard from the list", async ({ page }) => {
    const { store, add } = await mockDashboardApi(page);
    add(ME, "ops");

    await gotoAndWaitForLoad(page, "/studio/dashboards");
    await expect(page.getByText("ops", { exact: true })).toBeVisible();

    const modal = page.locator(".modal");
    await page.getByTestId("dashboard-rename").click();
    await modal.getByLabel(/^Name/).fill("sales");
    await page.getByTestId("modal-button-confirm").click();
    await expect(page.getByText("sales", { exact: true })).toBeVisible();
    expect([...store.keys()]).toEqual(["testuser/sales"]);

    await page.getByTestId("dashboard-remove").click();
    await page.getByTestId("modal-button-confirm").click();
    await expect(page.getByTestId("dashboards-empty")).toBeVisible();
    expect(store.size).toBe(0);
  });

  test("opens the home dashboard at the start page", async ({ page }) => {
    const { add, home } = await mockDashboardApi(page);
    add(ME, "ops");

    await gotoAndWaitForLoad(page, "/studio/dashboards");
    const star = page.getByTestId("dashboard-home");
    await expect(star).toHaveAttribute("aria-pressed", "false");
    await star.click();
    await expect(star).toHaveAttribute("aria-pressed", "true");
    expect(home.key).toBe("testuser/ops");

    await gotoAndWaitForLoad(page, "/studio/");
    await expect(page).toHaveURL(/\/studio\/dashboards\/testuser\/ops$/);
    await expect(page.getByTestId("dashboard-add-widget")).toBeEnabled();
    // The logo leads to the start page, i.e. back to this dashboard, which
    // stays open.
    await page.locator(".logo-wrapper").click();
    await expect(page).toHaveURL(/\/studio\/dashboards\/testuser\/ops$/);
    await expect(page.getByTestId("dashboard-add-widget")).toBeEnabled();
    // The databases stay one click away.
    await page.getByRole("link", { name: "Databases" }).click();
    await expect(page).toHaveURL(/\/studio\/db$/);

    await gotoAndWaitForLoad(page, "/studio/dashboards");
    await page.getByTestId("dashboard-home").click();
    await expect(page.getByTestId("dashboard-home")).toHaveAttribute(
      "aria-pressed",
      "false",
    );
    await gotoAndWaitForLoad(page, "/studio/");
    await expect(page).toHaveURL(/\/studio\/db$/);
  });

  test("shows a dashboard shared read only without editing", async ({
    page,
  }) => {
    const { add, store, forbidden } = await mockDashboardApi(page);
    add("ann", "ops", [[ME, "read"]]);

    await gotoAndWaitForLoad(page, "/studio/dashboards");
    await expect(page.getByText("ann/ops", { exact: true })).toBeVisible();
    await expect(page.getByTestId("dashboard-role")).toHaveText("Read only");
    await expect(page.getByTestId("dashboard-rename")).toHaveCount(0);

    await page.getByText("ann/ops", { exact: true }).click();
    await expect(page).toHaveURL(/\/studio\/dashboards\/ann\/ops$/);
    await expect(page.getByTestId("dashboard-read-only")).toBeVisible();
    await expect(page.getByTestId("dashboard-add-widget")).toHaveCount(0);

    await gotoAndWaitForLoad(page, "/studio/dashboards");
    await page.getByTestId("dashboard-leave").click();
    await page.getByTestId("modal-button-confirm").click();
    await expect(page.getByTestId("dashboards-empty")).toBeVisible();
    expect(store.get("ann/ops")!.users.has(ME)).toBe(false);
    expect(forbidden).toEqual([]);
  });

  test("shares a dashboard with another user", async ({ page }) => {
    const { add, store } = await mockDashboardApi(page);
    add(ME, "ops");

    await gotoAndWaitForLoad(page, "/studio/dashboards");
    await page.getByTestId("dashboard-users-toggle").click();
    const users = page.getByTestId("dashboard-users");
    await expect(users.locator(".username")).toHaveText([ME]);

    await users.locator(".add-button").click();
    const modal = page.locator(".modal");
    await modal.getByLabel(/^Username/).fill("bob");
    await modal.getByLabel(/^Role/).selectOption("write");
    await page.getByTestId("modal-button-confirm").click();
    await expect(users.locator(".username")).toHaveText([ME, "bob"]);
    expect(store.get("testuser/ops")!.users.get("bob")).toBe("write");

    await users.locator(".remove-button").click();
    await page.getByTestId("modal-button-confirm").click();
    await expect(users.locator(".username")).toHaveText([ME]);
  });

  test("makes an own dashboard public", async ({ page }) => {
    const { add, store } = await mockDashboardApi(page);
    add(ME, "ops");

    await gotoAndWaitForLoad(page, "/studio/dashboards");
    const toggle = page.getByTestId("dashboard-public");
    await expect(toggle).toHaveAttribute("aria-pressed", "false");
    await toggle.click();
    await expect(page.locator(".modal")).toContainText("All users can view it");
    await page.getByTestId("modal-button-confirm").click();
    await expect(toggle).toHaveAttribute("aria-pressed", "true");
    expect(store.get("testuser/ops")!.public).toBe(true);

    await page.getByRole("link", { name: "ops", exact: true }).click();
    await expect(page.getByTestId("dashboard-public-badge")).toBeVisible();
  });

  test("shows public dashboards of others read only, and as home", async ({
    page,
  }) => {
    const { add, home, forbidden } = await mockDashboardApi(page);
    add(ME, "mine");
    add("ann", "kpi", [], true);

    await gotoAndWaitForLoad(page, "/studio/dashboards");
    const section = page.getByTestId("dashboards-public");
    await expect(section.getByText("ann/kpi", { exact: true })).toBeVisible();
    await expect(section.getByTestId("dashboard-public-badge")).toBeVisible();
    await expect(section.getByTestId("dashboard-users-toggle")).toHaveCount(0);
    await expect(section.getByTestId("dashboard-leave")).toHaveCount(0);
    await expect(page.getByTestId("dashboards-shared")).toHaveCount(0);

    await section.getByTestId("dashboard-home").click();
    await expect(section.getByTestId("dashboard-home")).toHaveAttribute(
      "aria-pressed",
      "true",
    );
    expect(home.key).toBe("ann/kpi");

    await gotoAndWaitForLoad(page, "/studio/");
    await expect(page).toHaveURL(/\/studio\/dashboards\/ann\/kpi$/);
    await expect(page.getByTestId("dashboard-read-only")).toBeVisible();
    await expect(page.getByTestId("dashboard-public-badge")).toBeVisible();
    await expect(page.getByTestId("dashboard-add-widget")).toHaveCount(0);
    expect(forbidden).toEqual([]);
  });
});

/** A dashboard of another user with one widget on bob/sales. */
const WIDGET_CONFIG = JSON.stringify({
  version: 2,
  widgets: {
    w1: {
      id: "w1",
      title: "People",
      owner: "bob",
      db: "sales",
      rows: {
        exec: [{ id: "r1", query: "search().elements()" }],
        context: [],
      },
      view: "table",
    },
  },
  layout: [{ id: "w1" }],
});

test.describe("Admin dashboards", () => {
  test("manages every user's dashboard with admin access", async ({ page }) => {
    await mockLogin(page);
    await mockAdminUserStatus(page);
    const added: string[] = [];
    const userCalls: string[] = [];
    await page.route(DASHBOARD_API, (route) => {
      userCalls.push(route.request().url());
      return route.fulfill({ status: 404 });
    });
    await page.route(ADMIN_DASHBOARD_API, async (route) => {
      const url = new URL(route.request().url());
      const path = url.pathname.split("/admin/dashboard/")[1] ?? "";
      const json = (body: unknown, status = 200) =>
        route.fulfill({
          status,
          contentType: "application/json",
          body: JSON.stringify(body),
        });
      if (path === "list") {
        return json([
          {
            owner: "ann",
            name: "kpi",
            updated: 1_700_000_000,
            role: "admin",
            public: false,
            member: true,
          },
        ]);
      }
      if (path.endsWith("/add")) {
        added.push(path);
        return json("", 201);
      }
      return json({
        owner: "ann",
        name: path.split("/")[1],
        config: WIDGET_CONFIG,
        updated: 1_700_000_000,
        role: "admin",
        public: false,
      });
    });
    const execs: string[] = [];
    await page.route(ADMIN_DB_EXEC_API, (route) => {
      execs.push(new URL(route.request().url()).pathname);
      return route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify([{ result: 2, elements: ELEMENTS }]),
      });
    });

    await gotoAndWaitForLoad(page, "/studio/admin/dashboards");
    const table = page.getByTestId("dashboards-all");
    await expect(table.getByText("ann/kpi", { exact: true })).toBeVisible();
    await expect(table.getByTestId("dashboard-home")).toHaveCount(0);
    await expect(table.getByTestId("dashboard-public")).toBeVisible();

    // A dashboard for a user needs the user.
    await page.getByTestId("dashboard-create").click();
    const modal = page.locator(".modal");
    await modal.getByLabel(/^Owner/).fill("carol");
    await modal.getByLabel(/^Name/).fill("ops");
    await page.getByTestId("modal-button-confirm").click();
    await expect(page).toHaveURL(/\/studio\/admin\/dashboards\/carol\/ops$/);
    expect(added).toEqual(["carol/ops/add"]);

    // Widgets run with admin access on the other user's database.
    await gotoAndWaitForLoad(page, "/studio/admin/dashboards");
    await page.getByText("ann/kpi", { exact: true }).click();
    await expect(page).toHaveURL(/\/studio\/admin\/dashboards\/ann\/kpi$/);
    await expect(page.getByTestId(testIds.RESULTS_TABLE_ROW)).toHaveCount(2);
    expect(execs[0]).toMatch(/\/api\/v1\/admin\/db\/bob\/sales\/exec$/);
    expect(userCalls).toEqual([]);
  });
});
