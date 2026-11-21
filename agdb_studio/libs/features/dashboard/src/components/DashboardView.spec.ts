import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { ref } from "vue";

const { store, fetchDatabases, leaveGuard } = vi.hoisted(() => ({
  store: {
    current: undefined as unknown as import("vue").Ref<unknown>,
    dirty: undefined as unknown as import("vue").Ref<boolean>,
    saving: undefined as unknown as import("vue").Ref<boolean>,
    saveFailed: undefined as unknown as import("vue").Ref<boolean>,
    loadDashboard: vi.fn(),
    saveDashboard: vi.fn(),
    closeDashboard: vi.fn(),
    settleSaves: vi.fn(),
    addWidget: vi.fn(),
    updateWidget: vi.fn(),
    removeWidget: vi.fn(),
    restoreWidget: vi.fn(),
    moveWidget: vi.fn(),
    resizeWidget: vi.fn(),
  },
  fetchDatabases: vi.fn(),
  leaveGuard: {
    fn: undefined as undefined | (() => Promise<boolean>),
    update: undefined as
      | undefined
      | ((
          to: { params: { name?: string } },
          from: { params: { name?: string } },
        ) => boolean | Promise<boolean>),
  },
}));

vi.mock("../composables/dashboardStore", () => ({
  useDashboardStore: () => store,
}));
vi.mock("@agdb-studio/db/src/composables/dbStore", () => ({
  useDbStore: () => ({ fetchDatabases }),
}));
vi.mock("vue-router", () => ({
  onBeforeRouteLeave: (fn: typeof leaveGuard.fn) => {
    leaveGuard.fn = fn;
  },
  onBeforeRouteUpdate: (fn: typeof leaveGuard.update) => {
    leaveGuard.update = fn;
  },
}));

import DashboardView from "./DashboardView.vue";
import {
  addToDocument,
  createWidget,
  emptyDocument,
  NewerDashboardError,
} from "../utils/dashboardDocument";

const WidgetStub = {
  name: "DashboardWidget",
  props: ["widget", "size", "first", "last", "refreshKey"],
  emits: ["edit", "remove", "move", "resize", "update"],
  template: '<div class="widget-stub" />',
};
const EditorStub = {
  name: "WidgetEditor",
  props: ["widget", "header"],
  emits: ["done", "cancel"],
  template: '<div class="editor-stub" />',
};

const stubs = {
  DashboardWidget: WidgetStub,
  WidgetEditor: EditorStub,
  SpinnerIcon: { template: '<div class="spinner" />' },
};

const widgets = [
  createWidget({ owner: "o", db: "d" }),
  createWidget({ owner: "o", db: "d" }),
];
const document = () => {
  const doc = emptyDocument();
  addToDocument(doc, widgets);
  doc.layout[1]!.cols = 4;
  return doc;
};

const mounted: ReturnType<typeof mount>[] = [];
afterEach(() => mounted.splice(0).forEach((w) => w.unmount()));

const mountView = async () => {
  const wrapper = mount(DashboardView, {
    props: { name: "ops" },
    global: { stubs },
  });
  mounted.push(wrapper);
  await flushPromises();
  return wrapper;
};

const click = async (wrapper: ReturnType<typeof mount>, id: string) => {
  await wrapper.find(`[data-testid="${id}"]`).trigger("click");
  await flushPromises();
};

describe("DashboardView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    store.current = ref({ name: "ops", document: document() });
    store.dirty = ref(false);
    store.saving = ref(false);
    store.saveFailed = ref(false);
    store.loadDashboard.mockResolvedValue(undefined);
    store.saveDashboard.mockResolvedValue(undefined);
    store.settleSaves.mockResolvedValue(undefined);
    fetchDatabases.mockResolvedValue(undefined);
    vi.spyOn(window, "confirm").mockReturnValue(true);
  });

  it("loads the dashboard and shows its widgets in order", async () => {
    const wrapper = await mountView();
    expect(store.loadDashboard).toHaveBeenCalledWith("ops");
    const items = wrapper.findAllComponents(WidgetStub);
    expect(items).toHaveLength(2);
    expect(items[0]!.props("first")).toBe(true);
    expect(items[1]!.props("last")).toBe(true);
    expect(items[1]!.props("size")).toMatchObject({ cols: 4, rows: 1 });
  });

  it("shows only the latest load when the name changes quickly", async () => {
    let failSlow: (error: unknown) => void = () => {};
    const wrapper = await mountView();
    store.loadDashboard.mockImplementationOnce(
      () => new Promise((_, reject) => (failSlow = reject)),
    );
    await wrapper.setProps({ name: "slow" });
    await wrapper.setProps({ name: "sales" });
    await flushPromises();
    failSlow(new Error("404"));
    await flushPromises();
    expect(wrapper.find('[data-testid="dashboard-failed"]').exists()).toBe(
      false,
    );
    expect(wrapper.findAllComponents(WidgetStub)).toHaveLength(2);
  });

  it("reloads when the name changes", async () => {
    const wrapper = await mountView();
    await wrapper.setProps({ name: "sales" });
    await flushPromises();
    expect(store.loadDashboard).toHaveBeenLastCalledWith("sales");
  });

  it("shows a load failure and an empty dashboard", async () => {
    store.loadDashboard.mockRejectedValueOnce(new Error("404"));
    const failed = await mountView();
    expect(failed.find('[data-testid="dashboard-failed"]').text()).toBe(
      "The dashboard could not be loaded.",
    );
    store.loadDashboard.mockRejectedValueOnce(new NewerDashboardError(3));
    const newer = await mountView();
    expect(newer.find('[data-testid="dashboard-failed"]').text()).toContain(
      "Update Studio",
    );

    store.current.value = {
      name: "ops",
      document: emptyDocument(),
    };
    const empty = await mountView();
    expect(empty.find('[data-testid="dashboard-empty"]').text()).toContain(
      "Add widget",
    );
    // Closed meanwhile (e.g. deleted from another view): nothing to show.
    store.current.value = undefined;
    await flushPromises();
    expect(empty.find('[data-testid="dashboard-empty"]').exists()).toBe(true);
  });

  it("refreshes all widgets", async () => {
    const wrapper = await mountView();
    await click(wrapper, "dashboard-refresh");
    expect(wrapper.findComponent(WidgetStub).props("refreshKey")).toBe(1);
  });

  it("edits widgets through the store", async () => {
    const wrapper = await mountView();
    const item = wrapper.findComponent(WidgetStub);
    item.vm.$emit("move", 1);
    expect(store.moveWidget).toHaveBeenCalledWith(widgets[0]!.id, 1);
    item.vm.$emit("resize", { cols: 3, rows: 2 });
    expect(store.resizeWidget).toHaveBeenCalledWith(widgets[0]!.id, {
      cols: 3,
      rows: 2,
    });
    const shown = { ...widgets[0]!, view: "table" };
    item.vm.$emit("update", shown);
    expect(store.updateWidget).toHaveBeenCalledWith(shown);

    item.vm.$emit("edit");
    await flushPromises();
    expect(fetchDatabases).toHaveBeenCalled();
    const editor = () => wrapper.findComponent(EditorStub);
    expect(editor().props("widget")).toEqual(widgets[0]);
    expect(editor().props("header")).toBe("Edit widget");
    editor().vm.$emit("done", widgets[0]);
    await flushPromises();
    expect(store.updateWidget).toHaveBeenLastCalledWith(widgets[0]);
    expect(editor().exists()).toBe(false);

    fetchDatabases.mockRejectedValueOnce(new Error("offline"));
    await click(wrapper, "dashboard-add-widget");
    expect(editor().props("header")).toBe("Add widget");
    const created = editor().props("widget");
    editor().vm.$emit("done", created);
    await flushPromises();
    expect(store.addWidget).toHaveBeenCalledWith(created);
    expect(store.updateWidget).toHaveBeenCalledTimes(2);

    await click(wrapper, "dashboard-add-widget");
    editor().vm.$emit("cancel");
    await flushPromises();
    expect(editor().exists()).toBe(false);
    expect(store.addWidget).toHaveBeenCalledTimes(1);
  });

  it("offers to undo a removal for a while", async () => {
    vi.useFakeTimers();
    try {
      const removed = { widget: widgets[0], layout: {}, index: 0 };
      store.removeWidget.mockReturnValue(removed);
      const wrapper = await mountView();
      const item = wrapper.findComponent(WidgetStub);
      const banner = () => wrapper.find('[data-testid="dashboard-removed"]');
      item.vm.$emit("remove");
      await flushPromises();
      expect(store.removeWidget).toHaveBeenCalledWith(widgets[0]!.id);
      expect(banner().text()).toContain(widgets[0]!.title);
      await click(wrapper, "dashboard-undo");
      expect(store.restoreWidget).toHaveBeenCalledWith(removed);
      expect(banner().exists()).toBe(false);

      item.vm.$emit("remove");
      await flushPromises();
      vi.advanceTimersByTime(8000);
      await flushPromises();
      expect(banner().exists()).toBe(false);
      expect(store.restoreWidget).toHaveBeenCalledTimes(1);

      store.removeWidget.mockReturnValue({
        ...removed,
        widget: { ...widgets[0], title: "" },
      });
      item.vm.$emit("remove");
      await flushPromises();
      expect(banner().text()).toContain('Removed "Untitled"');
    } finally {
      vi.useRealTimers();
    }
  });

  it("shows saving and failed saves with a retry", async () => {
    const wrapper = await mountView();
    const has = (id: string) => wrapper.find(`[data-testid="${id}"]`).exists();
    expect(has("dashboard-saving")).toBe(false);
    store.saving.value = true;
    await flushPromises();
    expect(has("dashboard-saving")).toBe(true);
    store.saving.value = false;
    store.saveFailed.value = true;
    await flushPromises();
    expect(has("dashboard-saving")).toBe(false);
    expect(has("dashboard-unsaved")).toBe(true);
    await click(wrapper, "dashboard-retry");
    expect(store.saveDashboard).toHaveBeenCalled();
  });

  it("asks before leaving with edits that failed to save", async () => {
    await mountView();
    store.dirty.value = true;
    expect(await leaveGuard.fn?.()).toBe(true);
    expect(window.confirm).not.toHaveBeenCalled();
    expect(store.closeDashboard).toHaveBeenCalledTimes(1);
    store.saveFailed.value = true;
    vi.mocked(window.confirm).mockReturnValueOnce(false);
    expect(await leaveGuard.fn?.()).toBe(false);
    expect(store.closeDashboard).toHaveBeenCalledTimes(1);
  });

  it("waits for a running save before deciding to leave", async () => {
    await mountView();
    let finishSave: () => void = () => {};
    store.settleSaves.mockImplementationOnce(
      () =>
        new Promise<void>((resolve) => {
          finishSave = resolve;
        }),
    );
    const leaving = leaveGuard.fn?.();
    await flushPromises();
    expect(window.confirm).not.toHaveBeenCalled();
    // The running save fails: its edits would be lost.
    store.saveFailed.value = true;
    vi.mocked(window.confirm).mockReturnValueOnce(false);
    finishSave();
    expect(await leaving).toBe(false);
    expect(window.confirm).toHaveBeenCalledTimes(1);
  });

  it("asks before switching to another dashboard with edits that failed to save", async () => {
    await mountView();
    const at = (name: string) => ({ params: { name } });
    store.saveFailed.value = true;
    expect(leaveGuard.update?.(at("ops"), at("ops"))).toBe(true);
    expect(window.confirm).not.toHaveBeenCalled();
    vi.mocked(window.confirm).mockReturnValueOnce(false);
    expect(await leaveGuard.update?.(at("other"), at("ops"))).toBe(false);
  });

  it("warns before closing the tab with unsaved edits", async () => {
    await mountView();
    const clean = new Event("beforeunload", { cancelable: true });
    window.dispatchEvent(clean);
    expect(clean.defaultPrevented).toBe(false);
    store.dirty.value = true;
    const dirty = new Event("beforeunload", { cancelable: true });
    window.dispatchEvent(dirty);
    expect(dirty.defaultPrevented).toBe(true);
  });
});
