import { beforeEach, describe, expect, it, vi } from "vitest";

const { client } = vi.hoisted(() => ({
  client: {
    value: {
      dashboard_list: vi.fn(),
      dashboard_get: vi.fn(),
      dashboard_add: vi.fn(),
      dashboard_save: vi.fn(),
      dashboard_rename: vi.fn(),
      dashboard_remove: vi.fn(),
      dashboard_user_remove: vi.fn(),
      user_home_dashboard: vi.fn(),
      user_home_dashboard_set: vi.fn(),
      user_home_dashboard_clear: vi.fn(),
      dashboard_public: vi.fn(),
      admin_dashboard_list: vi.fn(),
      admin_dashboard_get: vi.fn(),
      admin_dashboard_add: vi.fn(),
      admin_dashboard_save: vi.fn(),
      admin_dashboard_rename: vi.fn(),
      admin_dashboard_remove: vi.fn(),
      admin_dashboard_public: vi.fn(),
    },
  },
}));

vi.mock("@agdb-studio/api/src/api", () => ({ checkClient: vi.fn(), client }));
const { username } = vi.hoisted(() => ({
  username: { value: "me" as string | undefined },
}));
vi.mock("@agdb-studio/auth/src/account", () => ({
  useAccount: () => ({ username }),
}));

import { watch, type Ref } from "vue";
import { useAdmin } from "@agdb-studio/router/src/admin";
import {
  adminMode,
  canEdit,
  canShare,
  dashboardLabel,
  dashboardRoute,
  isOwner,
  sameDashboard,
  useDashboardStore,
} from "./dashboardStore";
import {
  addToDocument,
  createWidget,
  emptyDocument,
  orderedWidgets,
  parseDocument,
  serializeDocument,
} from "../utils/dashboardDocument";

const widget = (id: string) => ({
  ...createWidget({ owner: "o", db: "d" }),
  id,
});
const config = (...ids: string[]) => {
  const doc = emptyDocument();
  addToDocument(doc, ids.map(widget));
  return serializeDocument(doc);
};
const ops = { owner: "me", name: "ops" };
const openIds = (store: ReturnType<typeof useDashboardStore>) =>
  orderedWidgets(store.current.value!.document).map(({ widget }) => widget.id);

describe("dashboardStore", () => {
  const store = useDashboardStore();

  beforeEach(() => {
    vi.clearAllMocks();
    username.value = "me";
    store.closeDashboard();
    client.value.user_home_dashboard.mockResolvedValue([200, null]);
    client.value.dashboard_list.mockResolvedValue([
      200,
      [{ owner: "me", name: "ops", updated: 1, role: "admin" }],
    ]);
    client.value.dashboard_get.mockResolvedValue([
      200,
      {
        owner: "me",
        name: "ops",
        config: config("a", "b", "c"),
        updated: 1,
        role: "admin",
      },
    ]);
  });

  it("fetches the dashboard list", async () => {
    await store.fetchDashboards();
    expect(store.dashboards.value).toEqual([
      { owner: "me", name: "ops", updated: 1, role: "admin" },
    ]);
  });

  it("loads a dashboard", async () => {
    const open = await store.loadDashboard(ops);
    expect(client.value.dashboard_get).toHaveBeenCalledWith(ops);
    expect(Object.keys(open.document.widgets)).toEqual(["a", "b", "c"]);
    expect(store.dirty.value).toBe(false);
  });

  it("closes the open dashboard when another one loads, and keeps only the latest load", async () => {
    await store.loadDashboard(ops);
    let resolveSlow: (value: unknown) => void = () => {};
    client.value.dashboard_get.mockImplementationOnce(
      () => new Promise((resolve) => (resolveSlow = resolve)),
    );
    const slow = store.loadDashboard({ owner: "me", name: "slow" });
    // Edits can't go to the previous dashboard while this one loads.
    expect(store.current.value).toBeUndefined();
    await store.loadDashboard(ops);
    resolveSlow([200, { name: "slow", config: config("x"), updated: 1 }]);
    await slow;
    expect(store.current.value?.name).toBe("ops");

    client.value.dashboard_get.mockRejectedValueOnce(new Error("404"));
    await expect(
      store.loadDashboard({ owner: "me", name: "gone" }),
    ).rejects.toThrow("404");
    expect(store.current.value).toBeUndefined();
  });

  it("saves each edit at once, one save at a time", async () => {
    await store.loadDashboard(ops);
    let finishSave: (value: unknown) => void = () => {};
    client.value.dashboard_save.mockImplementationOnce(
      () => new Promise((resolve) => (finishSave = resolve)),
    );
    store.moveWidget("a", 1);
    expect(store.saving.value).toBe(true);
    store.removeWidget("c");
    store.resizeWidget("a", { cols: 1, rows: 1 });
    // The later edits wait for the running save, then go in one more.
    expect(client.value.dashboard_save).toHaveBeenCalledTimes(1);
    expect(store.dirty.value).toBe(true);
    finishSave(undefined);
    await store.saveDashboard();
    expect(client.value.dashboard_save).toHaveBeenCalledTimes(2);
    const [body] = client.value.dashboard_save.mock.calls[1]!;
    expect(orderedWidgets(parseDocument(body.config)).length).toBe(2);
    expect(store.dirty.value).toBe(false);
    expect(store.saving.value).toBe(false);
  });

  it("keeps edits whose save failed until a retry", async () => {
    await store.loadDashboard(ops);
    client.value.dashboard_save.mockRejectedValueOnce(new Error("500"));
    store.moveWidget("a", 1);
    await vi.waitFor(() => expect(store.saving.value).toBe(false));
    expect(store.saveFailed.value).toBe(true);
    expect(store.dirty.value).toBe(true);
    await store.saveDashboard();
    expect(store.saveFailed.value).toBe(false);
    expect(store.dirty.value).toBe(false);
  });

  it("finishes a save of a dashboard closed meanwhile without touching the next one", async () => {
    await store.loadDashboard(ops);
    let failSave: (error: unknown) => void = () => {};
    client.value.dashboard_save.mockImplementationOnce(
      () => new Promise((_, reject) => (failSave = reject)),
    );
    store.moveWidget("a", 1);
    // The load waits for the running save, so it reads what was saved.
    const loading = store.loadDashboard({ owner: "me", name: "sales" });
    await Promise.resolve();
    expect(store.current.value).toBeUndefined();
    expect(client.value.dashboard_get).toHaveBeenCalledTimes(1);
    failSave(new Error("500"));
    await loading;
    expect(client.value.dashboard_get).toHaveBeenCalledTimes(2);
    expect(store.current.value?.name).toBe("sales");
    expect(store.saveFailed.value).toBe(false);
    expect(store.dirty.value).toBe(false);
  });

  it("saves an edit made while the save loop is finishing", async () => {
    await store.loadDashboard(ops);
    let edited = false;
    // Edits as the loop ends: it has seen the queue empty already.
    const stop = watch(
      store.saving,
      (saving) => {
        if (saving || edited) return;
        edited = true;
        store.moveWidget("a", 0);
      },
      { flush: "sync" },
    );
    store.moveWidget("a", 1);
    await store.settleSaves();
    stop();
    expect(client.value.dashboard_save).toHaveBeenCalledTimes(2);
    expect(store.dirty.value).toBe(false);
  });

  it("appends only after a running save", async () => {
    await store.loadDashboard(ops);
    let finishSave: (value: unknown) => void = () => {};
    client.value.dashboard_save.mockImplementationOnce(
      () => new Promise((resolve) => (finishSave = resolve)),
    );
    store.moveWidget("a", 1);
    const appending = store.appendWidgets(ops, [widget("d")]);
    await Promise.resolve();
    expect(client.value.dashboard_get).toHaveBeenCalledTimes(1);
    finishSave(undefined);
    await appending;
    expect(client.value.dashboard_get).toHaveBeenCalledTimes(2);
  });

  it("undoes a removal in place", async () => {
    await store.loadDashboard(ops);
    store.resizeWidget("b", { cols: 3, rows: 2 });
    const removed = store.removeWidget("b")!;
    expect(openIds(store)).toEqual(["a", "c"]);
    store.restoreWidget(removed);
    expect(openIds(store)).toEqual(["a", "b", "c"]);
    expect(store.current.value!.document.layout[1]).toEqual({
      id: "b",
      cols: 3,
      rows: 2,
    });
    // Already back: nothing to do.
    const saves = client.value.dashboard_save.mock.calls.length;
    store.restoreWidget(removed);
    expect(client.value.dashboard_save).toHaveBeenCalledTimes(saves);
    expect(store.removeWidget("missing")).toBeUndefined();
  });

  it("creates an empty dashboard", async () => {
    expect(await store.createDashboard("new")).toEqual({
      owner: "me",
      name: "new",
    });
    expect(client.value.dashboard_add).toHaveBeenCalledWith({
      owner: "me",
      name: "new",
      config: serializeDocument(emptyDocument()),
    });
    expect(client.value.dashboard_list).toHaveBeenCalled();
  });

  it("edits widgets and saves", async () => {
    await store.loadDashboard(ops);
    // Edits that change nothing leave the dashboard saved.
    store.moveWidget("a", -1);
    store.resizeWidget("missing", { cols: 1, rows: 1 });
    store.updateWidget(widget("missing"));
    expect(store.dirty.value).toBe(false);

    store.moveWidget("a", 1);
    store.resizeWidget("b", { cols: 4, rows: 2 });
    store.removeWidget("c");
    store.addWidget(widget("d"));
    store.updateWidget({ ...widget("b"), title: "Edited" });
    expect(client.value.dashboard_save).toHaveBeenCalled();
    expect(openIds(store)).toEqual(["b", "a", "d"]);
    const doc = store.current.value!.document;
    // A content edit keeps the widget's size.
    expect(doc.widgets.b!.title).toBe("Edited");
    expect(doc.layout[0]).toEqual({ id: "b", cols: 4, rows: 2 });

    await store.saveDashboard();
    expect(client.value.dashboard_save).toHaveBeenCalledWith({
      ...ops,
      config: expect.stringContaining('"id":"d"'),
    });
    expect(store.dirty.value).toBe(false);
  });

  it("ignores edits and saves without an open dashboard", async () => {
    store.addWidget(widget("x"));
    store.removeWidget("x");
    store.updateWidget(widget("x"));
    store.moveWidget("x", 1);
    store.resizeWidget("x", { cols: 1, rows: 1 });
    await store.saveDashboard();
    expect(client.value.dashboard_save).not.toHaveBeenCalled();
    expect(store.dirty.value).toBe(false);
  });

  it("renames the open and other dashboards", async () => {
    await store.loadDashboard(ops);
    await store.renameDashboard({ owner: "me", name: "other" }, "x");
    expect(store.current.value?.name).toBe("ops");
    await store.renameDashboard(ops, "sales");
    expect(client.value.dashboard_rename).toHaveBeenCalledWith({
      ...ops,
      new_name: "sales",
    });
    expect(store.current.value?.name).toBe("sales");
  });

  it("renames only after a running save", async () => {
    await store.loadDashboard(ops);
    let finishSave: (value: unknown) => void = () => {};
    client.value.dashboard_save.mockImplementationOnce(
      () => new Promise((resolve) => (finishSave = resolve)),
    );
    store.moveWidget("a", 1);
    const renaming = store.renameDashboard(ops, "sales");
    await Promise.resolve();
    expect(client.value.dashboard_rename).not.toHaveBeenCalled();
    finishSave(undefined);
    await renaming;
    expect(client.value.dashboard_rename).toHaveBeenCalledTimes(1);
    expect(store.current.value?.name).toBe("sales");
  });

  it("removes only after a running save and drops the queued ones", async () => {
    await store.loadDashboard(ops);
    let finishSave: (value: unknown) => void = () => {};
    client.value.dashboard_save.mockImplementationOnce(
      () => new Promise((resolve) => (finishSave = resolve)),
    );
    store.moveWidget("a", 1);
    // Queued behind the running save: it would create the dashboard again.
    store.moveWidget("a", 0);
    const removing = store.removeDashboard(ops);
    await Promise.resolve();
    expect(client.value.dashboard_remove).not.toHaveBeenCalled();
    finishSave(undefined);
    await removing;
    expect(client.value.dashboard_save).toHaveBeenCalledTimes(1);
    expect(client.value.dashboard_remove).toHaveBeenCalledTimes(1);
    expect(store.current.value).toBeUndefined();
  });

  it("leaves only after a running save and drops the queued ones", async () => {
    const shared = { owner: "ann", name: "ops" };
    await store.loadDashboard(shared);
    let finishSave: (value: unknown) => void = () => {};
    client.value.dashboard_save.mockImplementationOnce(
      () => new Promise((resolve) => (finishSave = resolve)),
    );
    store.moveWidget("a", 1);
    // Queued behind the running save: it would be rejected after leaving.
    store.moveWidget("a", 0);
    const leaving = store.leaveDashboard(shared);
    await Promise.resolve();
    expect(client.value.dashboard_user_remove).not.toHaveBeenCalled();
    finishSave(undefined);
    await leaving;
    expect(client.value.dashboard_save).toHaveBeenCalledTimes(1);
    expect(client.value.dashboard_user_remove).toHaveBeenCalledWith({
      ...shared,
      username: "me",
    });
    expect(store.current.value).toBeUndefined();
  });

  it("does nothing when a dashboard keeps its name", async () => {
    await store.renameDashboard(ops, "ops");
    expect(client.value.dashboard_rename).not.toHaveBeenCalled();
  });

  it("removes dashboards, closing the open one", async () => {
    await store.loadDashboard(ops);
    await store.removeDashboard({ owner: "me", name: "other" });
    expect(store.current.value).toBeDefined();
    store.addWidget(widget("x"));
    await store.removeDashboard(ops);
    expect(client.value.dashboard_remove).toHaveBeenCalledWith(ops);
    expect(store.current.value).toBeUndefined();
    expect(store.dirty.value).toBe(false);
  });

  it("appends a widget to a saved dashboard", async () => {
    await store.appendWidgets(ops, [widget("z")]);
    const [body] = client.value.dashboard_save.mock.calls[0]!;
    expect(body).toMatchObject(ops);
    expect(parseDocument(body.config)).toEqual(
      parseDocument(config("a", "b", "c", "z")),
    );
  });

  it("does not let a pin overwrite a dashboard saved by a newer Studio", async () => {
    client.value.dashboard_get.mockResolvedValueOnce([
      200,
      { ...ops, config: '{"version":3,"widgets":{}}', updated: 1 },
    ]);
    await expect(store.appendWidgets(ops, [widget("z")])).rejects.toThrow(
      "newer agdb Studio",
    );
    expect(client.value.dashboard_save).not.toHaveBeenCalled();
  });

  it("opens a dashboard with the user's role", async () => {
    client.value.dashboard_get.mockResolvedValueOnce([
      200,
      {
        owner: "ann",
        name: "ops",
        config: config("a"),
        updated: 1,
        role: "read",
      },
    ]);
    const open = await store.loadDashboard({ owner: "ann", name: "ops" });
    expect(open.owner).toBe("ann");
    expect(open.role).toBe("read");
  });

  it("leaves a shared dashboard, forgetting it", async () => {
    const shared = { owner: "ann", name: "ops" };
    client.value.dashboard_get.mockResolvedValueOnce([
      200,
      { ...shared, config: config("a"), updated: 1, role: "read" },
    ]);
    await store.loadDashboard(shared);
    store.home.value = { ...shared };
    client.value.user_home_dashboard.mockResolvedValueOnce([200, null]);
    await store.leaveDashboard(shared);
    expect(client.value.dashboard_user_remove).toHaveBeenCalledWith({
      ...shared,
      username: "me",
    });
    expect(store.current.value).toBeUndefined();
    expect(store.home.value).toBeUndefined();
    expect(client.value.dashboard_list).toHaveBeenCalled();
  });

  it("keeps a public dashboard left as the home, as the server does", async () => {
    const open = { owner: "ann", name: "kpi" };
    await store.loadDashboard(ops);
    store.home.value = { ...open };
    client.value.user_home_dashboard.mockResolvedValueOnce([200, open]);
    await store.leaveDashboard(open);
    expect(store.home.value).toEqual(open);
    // Another dashboard stays open.
    expect(store.current.value?.name).toBe("ops");
  });

  it("fetches, sets and clears the home dashboard", async () => {
    client.value.user_home_dashboard.mockResolvedValueOnce([200, null]);
    expect(await store.fetchHome()).toBeUndefined();
    client.value.user_home_dashboard.mockResolvedValueOnce([200, ops]);
    expect(await store.fetchHome()).toEqual(ops);

    await store.setHome({ owner: "ann", name: "x" });
    expect(client.value.user_home_dashboard_set).toHaveBeenCalledWith({
      owner: "ann",
      name: "x",
    });
    expect(store.home.value).toEqual({ owner: "ann", name: "x" });

    await store.setHome();
    expect(client.value.user_home_dashboard_clear).toHaveBeenCalled();
    expect(store.home.value).toBeUndefined();
  });

  it("keeps the home on rename and drops it on remove", async () => {
    store.home.value = { ...ops };
    await store.renameDashboard(ops, "sales");
    expect(store.home.value).toEqual({ owner: "me", name: "sales" });
    await store.removeDashboard({ owner: "me", name: "sales" });
    expect(store.home.value).toBeUndefined();
  });

  it("derives permissions and labels", () => {
    expect(canEdit("read")).toBe(false);
    expect(canEdit("write")).toBe(true);
    expect(canEdit("admin")).toBe(true);
    expect(canShare("write")).toBe(false);
    expect(canShare("admin")).toBe(true);
    expect(isOwner(ops)).toBe(true);
    expect(isOwner({ owner: "ann", name: "ops" })).toBe(false);
    expect(dashboardLabel(ops)).toBe("ops");
    expect(dashboardLabel({ owner: "ann", name: "ops" })).toBe("ann/ops");
    expect(sameDashboard(ops, { ...ops })).toBe(true);
    expect(sameDashboard(ops, { owner: "ann", name: "ops" })).toBe(false);
    expect(sameDashboard(ops, undefined)).toBe(false);
  });

  it("sends an empty owner while the user is unknown", async () => {
    username.value = undefined;
    await store.createDashboard("new");
    expect(client.value.dashboard_add.mock.calls[0]![0]).toMatchObject({
      owner: "",
      name: "new",
    });
    await store.leaveDashboard(ops);
    expect(client.value.dashboard_user_remove).toHaveBeenCalledWith({
      ...ops,
      username: "",
    });
  });

  it("splits the list into own, shared and public dashboards", async () => {
    const summary = (owner: string, name: string, member: boolean) => ({
      owner,
      name,
      updated: 1,
      role: "read",
      public: !member,
      member,
    });
    client.value.dashboard_list.mockResolvedValueOnce([
      200,
      [
        summary("ann", "kpi", false),
        summary("ann", "sales", true),
        summary("me", "ops", true),
      ],
    ]);
    await store.fetchDashboards();
    const names = (list: { name: string }[]) => list.map((d) => d.name);
    expect(names(store.mine.value)).toEqual(["ops"]);
    expect(names(store.shared.value)).toEqual(["sales"]);
    expect(names(store.publicOnly.value)).toEqual(["kpi"]);
  });

  it("makes a dashboard public, updating the open one", async () => {
    client.value.dashboard_get.mockResolvedValueOnce([
      200,
      {
        ...ops,
        config: config(),
        updated: 1,
        role: "admin",
        public: false,
      },
    ]);
    await store.loadDashboard(ops);
    expect(store.current.value!.public).toBe(false);
    await store.setPublic(ops, true);
    expect(client.value.dashboard_public).toHaveBeenCalledWith({
      ...ops,
      is_public: true,
    });
    expect(store.current.value!.public).toBe(true);
    expect(client.value.dashboard_list).toHaveBeenCalled();
    await store.setPublic({ owner: "me", name: "other" }, false);
    expect(store.current.value!.public).toBe(true);
  });

  describe("in the admin view", () => {
    const setAdmin = (on: boolean) => {
      const { isAdmin, isAdminView } = useAdmin();
      (isAdmin as Ref<boolean>).value = on;
      (isAdminView as Ref<boolean>).value = on;
    };
    const ann = { owner: "ann", name: "sales" };

    beforeEach(() => {
      setAdmin(true);
      client.value.admin_dashboard_list.mockResolvedValue([200, []]);
      client.value.admin_dashboard_get.mockResolvedValue([
        200,
        {
          ...ann,
          config: config("a"),
          updated: 1,
          role: "admin",
          public: true,
        },
      ]);
    });

    it("needs both the admin view and an admin", () => {
      expect(adminMode()).toBe(true);
      const { isAdmin } = useAdmin();
      (isAdmin as Ref<boolean>).value = false;
      expect(adminMode()).toBe(false);
    });

    it("acts as the owner of every dashboard", () => {
      expect(isOwner(ann)).toBe(true);
      expect(dashboardLabel(ops)).toBe("me/ops");
      expect(dashboardRoute(ann)).toEqual({
        name: "admin-dashboard",
        params: ann,
      });
      setAdmin(false);
      expect(dashboardRoute(ann)).toEqual({ name: "dashboard", params: ann });
    });

    it("uses the admin routes", async () => {
      await store.fetchDashboards();
      expect(client.value.admin_dashboard_list).toHaveBeenCalled();
      expect(client.value.dashboard_list).not.toHaveBeenCalled();

      const open = await store.loadDashboard(ann);
      expect(client.value.admin_dashboard_get).toHaveBeenCalledWith(ann);
      expect(open.public).toBe(true);

      // Edits save at once.
      store.addWidget(widget("b"));
      await store.settleSaves();
      expect(client.value.admin_dashboard_save.mock.calls[0]![0]).toMatchObject(
        ann,
      );

      await store.appendWidgets(ann, [widget("c")]);
      expect(client.value.admin_dashboard_save).toHaveBeenCalledTimes(2);
      expect(client.value.dashboard_get).not.toHaveBeenCalled();
      expect(client.value.dashboard_save).not.toHaveBeenCalled();

      await store.setPublic(ann, false);
      expect(client.value.admin_dashboard_public).toHaveBeenCalledWith({
        ...ann,
        is_public: false,
      });

      await store.renameDashboard(ann, "kpi");
      expect(client.value.admin_dashboard_rename).toHaveBeenCalledWith({
        ...ann,
        new_name: "kpi",
      });

      await store.removeDashboard({ ...ann, name: "kpi" });
      expect(client.value.admin_dashboard_remove).toHaveBeenCalledWith({
        ...ann,
        name: "kpi",
      });
    });

    it("creates a dashboard for another user", async () => {
      expect(await store.createDashboard("new", "ann")).toEqual({
        owner: "ann",
        name: "new",
      });
      expect(client.value.admin_dashboard_add.mock.calls[0]![0]).toMatchObject({
        owner: "ann",
        name: "new",
      });
      // Without an owner it is the admin's own.
      expect(await store.createDashboard("mine")).toEqual({
        owner: "me",
        name: "mine",
      });
      expect(client.value.admin_dashboard_add).toHaveBeenCalledTimes(1);
      expect(client.value.dashboard_add.mock.calls[0]![0]).toMatchObject({
        owner: "me",
        name: "mine",
      });
    });
  });
});
