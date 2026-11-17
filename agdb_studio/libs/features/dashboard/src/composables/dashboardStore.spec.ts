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
    },
  },
}));

vi.mock("@agdb-studio/api/src/api", () => ({ checkClient: vi.fn(), client }));

import { useDashboardStore } from "./dashboardStore";
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
const openIds = (store: ReturnType<typeof useDashboardStore>) =>
  orderedWidgets(store.current.value!.document).map(({ widget }) => widget.id);

describe("dashboardStore", () => {
  const store = useDashboardStore();

  beforeEach(() => {
    vi.clearAllMocks();
    store.closeDashboard();
    client.value.dashboard_list.mockResolvedValue({
      data: [{ name: "ops", updated: 1 }],
    });
    client.value.dashboard_get.mockResolvedValue({
      data: { name: "ops", config: config("a", "b", "c"), updated: 1 },
    });
  });

  it("fetches the dashboard list", async () => {
    await store.fetchDashboards();
    expect(store.dashboards.value).toEqual([{ name: "ops", updated: 1 }]);
  });

  it("loads a dashboard", async () => {
    const open = await store.loadDashboard("ops");
    expect(client.value.dashboard_get).toHaveBeenCalledWith({ name: "ops" });
    expect(Object.keys(open.document.widgets)).toEqual(["a", "b", "c"]);
    expect(store.dirty.value).toBe(false);
  });

  it("closes the open dashboard when another one loads, and keeps only the latest load", async () => {
    await store.loadDashboard("ops");
    let resolveSlow: (value: unknown) => void = () => {};
    client.value.dashboard_get.mockImplementationOnce(
      () => new Promise((resolve) => (resolveSlow = resolve)),
    );
    const slow = store.loadDashboard("slow");
    // Edits can't go to the previous dashboard while this one loads.
    expect(store.current.value).toBeUndefined();
    await store.loadDashboard("ops");
    resolveSlow({ data: { name: "slow", config: config("x"), updated: 1 } });
    await slow;
    expect(store.current.value?.name).toBe("ops");

    client.value.dashboard_get.mockRejectedValueOnce(new Error("404"));
    await expect(store.loadDashboard("gone")).rejects.toThrow("404");
    expect(store.current.value).toBeUndefined();
  });

  it("keeps edits made while saving marked as unsaved", async () => {
    await store.loadDashboard("ops");
    let finishSave: (value: unknown) => void = () => {};
    client.value.dashboard_save.mockImplementationOnce(
      () => new Promise((resolve) => (finishSave = resolve)),
    );
    store.moveWidget("a", 1);
    const saving = store.saveDashboard();
    store.removeWidget("c");
    finishSave(undefined);
    await saving;
    expect(store.dirty.value).toBe(true);
  });

  it("creates an empty dashboard", async () => {
    await store.createDashboard("new");
    expect(client.value.dashboard_add).toHaveBeenCalledWith(
      { name: "new" },
      { config: serializeDocument(emptyDocument()) },
    );
    expect(client.value.dashboard_list).toHaveBeenCalled();
  });

  it("edits widgets and saves", async () => {
    await store.loadDashboard("ops");
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
    expect(openIds(store)).toEqual(["b", "a", "d"]);
    const doc = store.current.value!.document;
    // A content edit keeps the widget's size.
    expect(doc.widgets.b!.title).toBe("Edited");
    expect(doc.layout[0]).toEqual({ id: "b", cols: 4, rows: 2 });
    expect(store.dirty.value).toBe(true);

    await store.saveDashboard();
    expect(client.value.dashboard_save).toHaveBeenCalledWith(
      { name: "ops" },
      { config: expect.stringContaining('"id":"d"') },
    );
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
    await store.loadDashboard("ops");
    await store.renameDashboard("other", "x");
    expect(store.current.value?.name).toBe("ops");
    await store.renameDashboard("ops", "sales");
    expect(client.value.dashboard_rename).toHaveBeenCalledWith({
      name: "ops",
      new_name: "sales",
    });
    expect(store.current.value?.name).toBe("sales");
  });

  it("does nothing when a dashboard keeps its name", async () => {
    await store.renameDashboard("ops", "ops");
    expect(client.value.dashboard_rename).not.toHaveBeenCalled();
  });

  it("removes dashboards, closing the open one", async () => {
    await store.loadDashboard("ops");
    await store.removeDashboard("other");
    expect(store.current.value).toBeDefined();
    store.addWidget(widget("x"));
    await store.removeDashboard("ops");
    expect(client.value.dashboard_remove).toHaveBeenCalledWith({ name: "ops" });
    expect(store.current.value).toBeUndefined();
    expect(store.dirty.value).toBe(false);
  });

  it("appends a widget to a saved dashboard", async () => {
    await store.appendWidgets("ops", [widget("z")]);
    const [params, body] = client.value.dashboard_save.mock.calls[0]!;
    expect(params).toEqual({ name: "ops" });
    expect(parseDocument(body.config)).toEqual(
      parseDocument(config("a", "b", "c", "z")),
    );
  });

  it("does not let a pin overwrite a dashboard saved by a newer Studio", async () => {
    client.value.dashboard_get.mockResolvedValueOnce({
      data: { name: "ops", config: '{"version":3,"widgets":{}}', updated: 1 },
    });
    await expect(store.appendWidgets("ops", [widget("z")])).rejects.toThrow(
      "newer agdb Studio",
    );
    expect(client.value.dashboard_save).not.toHaveBeenCalled();
  });
});
