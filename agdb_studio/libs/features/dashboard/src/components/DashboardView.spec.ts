import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { ref } from "vue";

const { store, fetchDatabases, leaveGuard } = vi.hoisted(() => ({
  store: {
    current: undefined as unknown as import("vue").Ref<unknown>,
    dirty: undefined as unknown as import("vue").Ref<boolean>,
    loadDashboard: vi.fn(),
    saveDashboard: vi.fn(),
    closeDashboard: vi.fn(),
    addWidget: vi.fn(),
    updateWidget: vi.fn(),
    removeWidget: vi.fn(),
    moveWidget: vi.fn(),
    resizeWidget: vi.fn(),
  },
  fetchDatabases: vi.fn(),
  leaveGuard: {
    fn: undefined as undefined | (() => boolean),
    update: undefined as
      | undefined
      | ((
          to: { params: { name?: string } },
          from: { params: { name?: string } },
        ) => boolean),
  },
}));

vi.mock("../composables/dashboardStore", () => ({
  useDashboardStore: () => store,
}));
vi.mock("@agdb-studio/db/src/composables/dbStore", () => ({
  useDbStore: () => ({ fetchDatabases }),
}));
vi.mock("vue-router", () => ({
  onBeforeRouteLeave: (fn: () => boolean) => {
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
  props: ["widget", "size", "editing", "first", "last", "refreshKey"],
  emits: ["edit", "remove", "move", "resize"],
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
    store.loadDashboard.mockResolvedValue(undefined);
    store.saveDashboard.mockResolvedValue(undefined);
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
      "Edit",
    );
    await click(empty, "dashboard-edit");
    expect(empty.find('[data-testid="dashboard-empty"]').text()).not.toContain(
      "pin",
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

  it("edits widgets through the store and saves", async () => {
    const wrapper = await mountView();
    await click(wrapper, "dashboard-edit");
    const item = wrapper.findComponent(WidgetStub);
    expect(item.props("editing")).toBe(true);
    item.vm.$emit("remove");
    item.vm.$emit("move", 1);
    expect(store.removeWidget).toHaveBeenCalledWith(widgets[0]!.id);
    expect(store.moveWidget).toHaveBeenCalledWith(widgets[0]!.id, 1);
    item.vm.$emit("resize", { cols: 3, rows: 2 });
    expect(store.resizeWidget).toHaveBeenCalledWith(widgets[0]!.id, {
      cols: 3,
      rows: 2,
    });

    item.vm.$emit("edit");
    await flushPromises();
    expect(fetchDatabases).toHaveBeenCalled();
    const editor = () => wrapper.findComponent(EditorStub);
    expect(editor().props("widget")).toEqual(widgets[0]);
    expect(editor().props("header")).toBe("Edit widget");
    editor().vm.$emit("done", widgets[0]);
    await flushPromises();
    expect(store.updateWidget).toHaveBeenCalledWith(widgets[0]);
    expect(editor().exists()).toBe(false);

    fetchDatabases.mockRejectedValueOnce(new Error("offline"));
    await click(wrapper, "dashboard-add-widget");
    expect(editor().props("header")).toBe("Add widget");
    const created = editor().props("widget");
    editor().vm.$emit("done", created);
    await flushPromises();
    expect(store.addWidget).toHaveBeenCalledWith(created);
    expect(store.updateWidget).toHaveBeenCalledTimes(1);

    await click(wrapper, "dashboard-add-widget");
    editor().vm.$emit("cancel");
    await flushPromises();
    expect(editor().exists()).toBe(false);
    expect(store.addWidget).toHaveBeenCalledTimes(1);

    store.dirty.value = true;
    await flushPromises();
    expect(wrapper.find('[data-testid="dashboard-unsaved"]').exists()).toBe(
      true,
    );
    await click(wrapper, "dashboard-save");
    expect(store.saveDashboard).toHaveBeenCalled();
    expect(wrapper.find('[data-testid="dashboard-edit"]').exists()).toBe(true);
  });

  it("keeps edit mode when saving fails", async () => {
    store.saveDashboard.mockRejectedValueOnce(new Error("500"));
    store.dirty.value = true;
    const wrapper = await mountView();
    await click(wrapper, "dashboard-edit");
    await click(wrapper, "dashboard-save");
    expect(wrapper.find('[data-testid="dashboard-save"]').exists()).toBe(true);
  });

  it("asks before discarding edits on cancel", async () => {
    const wrapper = await mountView();
    await click(wrapper, "dashboard-edit");
    store.dirty.value = true;
    vi.mocked(window.confirm).mockReturnValueOnce(false);
    await click(wrapper, "dashboard-cancel");
    expect(wrapper.find('[data-testid="dashboard-cancel"]').exists()).toBe(
      true,
    );
    await click(wrapper, "dashboard-cancel");
    expect(store.loadDashboard).toHaveBeenCalledTimes(2);
    expect(wrapper.find('[data-testid="dashboard-edit"]').exists()).toBe(true);
  });

  it("asks before leaving with unsaved edits", async () => {
    await mountView();
    expect(leaveGuard.fn?.()).toBe(true);
    expect(window.confirm).not.toHaveBeenCalled();
    expect(store.closeDashboard).toHaveBeenCalledTimes(1);
    store.dirty.value = true;
    vi.mocked(window.confirm).mockReturnValueOnce(false);
    expect(leaveGuard.fn?.()).toBe(false);
    expect(store.closeDashboard).toHaveBeenCalledTimes(1);
  });

  it("asks before switching to another dashboard with unsaved edits", async () => {
    await mountView();
    const at = (name: string) => ({ params: { name } });
    store.dirty.value = true;
    expect(leaveGuard.update?.(at("ops"), at("ops"))).toBe(true);
    expect(window.confirm).not.toHaveBeenCalled();
    vi.mocked(window.confirm).mockReturnValueOnce(false);
    expect(leaveGuard.update?.(at("other"), at("ops"))).toBe(false);
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
