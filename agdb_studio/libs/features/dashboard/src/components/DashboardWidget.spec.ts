import { beforeEach, describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { ref, type Ref } from "vue";
import { useAdmin } from "@agdb-studio/router/src/admin";

const { data, observer } = vi.hoisted(() => ({
  data: {
    error: undefined as unknown as import("vue").Ref<string | undefined>,
    loading: undefined as unknown as import("vue").Ref<boolean>,
    revision: undefined as unknown as import("vue").Ref<number>,
    query: undefined as unknown as import("vue").Ref<
      { lastRun?: Date } | undefined
    >,
    run: vi.fn(),
    displayChanged: undefined as unknown as import("vue").Ref<boolean>,
    resetDisplay: vi.fn(),
    shownWidget: vi.fn(),
  },
  observer: {
    callback: undefined as
      undefined | ((entries: { isIntersecting: boolean }[]) => void),
  },
}));
vi.mock("../composables/widgetQuery", () => ({ useWidgetQuery: () => data }));
vi.mock("@vueuse/core", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@vueuse/core")>()),
  useIntersectionObserver: (
    _target: unknown,
    callback: typeof observer.callback,
  ) => {
    observer.callback = callback;
  },
}));
vi.mock("@agdb-studio/query/src/components/results/QueryResults.vue", () => ({
  default: {
    name: "QueryResults",
    props: { keepView: Boolean },
    template: '<div class="results" />',
  },
}));

import DashboardWidget from "./DashboardWidget.vue";
import { createWidget } from "../utils/dashboardDocument";

const stubs = {
  RouterLink: { props: ["to"], template: '<a class="link"><slot /></a>' },
  SpinnerIcon: { template: '<div class="spinner" />' },
};

const widget = createWidget({
  owner: "o",
  db: "d",
  title: "Users",
  rows: {
    exec: [
      { id: "r1", query: 'select().ids("users")' },
      { id: "r2", query: "select().ids(1)" },
    ],
    context: [{ id: "c1", query: "select().aliases()" }],
  },
});

const mountWidget = (props: Record<string, unknown>) =>
  mount(DashboardWidget, {
    props: { widget, size: { cols: 2, rows: 1 }, ...props },
    global: { stubs },
  });

describe("DashboardWidget", () => {
  beforeEach(() => {
    data.error = ref(undefined);
    data.loading = ref(false);
    data.revision = ref(1);
    data.query = ref({ lastRun: new Date() });
    data.run.mockClear();
    data.displayChanged = ref(false);
    data.resetDisplay.mockClear();
    data.shownWidget.mockReset();
    observer.callback = undefined;
  });

  it("shows the query results only while on screen", async () => {
    const wrapper = mountWidget({});
    expect(wrapper.text()).toContain("Users");
    expect(wrapper.text()).toContain("o/d");
    expect(wrapper.find(".results").exists()).toBe(false);
    observer.callback?.([{ isIntersecting: true }]);
    await wrapper.vm.$nextTick();
    expect(wrapper.find(".results").exists()).toBe(true);
    // The saved view stays even when a refresh changes the result shape.
    expect(
      wrapper.findComponent({ name: "QueryResults" }).props("keepView"),
    ).toBe(true);
    observer.callback?.([]);
    await wrapper.vm.$nextTick();
    expect(wrapper.find(".results").exists()).toBe(false);
  });

  it("maximizes over the page and shows the results", async () => {
    const wrapper = mountWidget({});
    const toggle = wrapper.find('[data-testid="widget-maximize"]');
    expect(toggle.attributes("aria-label")).toBe("Maximize");
    await toggle.trigger("click");
    expect(wrapper.find("section").classes()).toContain("maximized");
    expect(wrapper.find("section").attributes("style")).toBeUndefined();
    expect(wrapper.find(".results").exists()).toBe(true);
    expect(toggle.attributes("aria-label")).toBe("Restore");
    await toggle.trigger("click");
    expect(wrapper.find("section").classes()).not.toContain("maximized");
  });

  it("spans its grid size", () => {
    const wrapper = mountWidget({ size: { cols: 3, rows: 2 } });
    const style = wrapper.find("section").attributes("style");
    expect(style).toContain("span 3");
    expect(style).toContain("span 2");
  });

  it("shows errors and the first load", async () => {
    data.query.value = {};
    data.loading.value = true;
    const wrapper = mountWidget({});
    expect(wrapper.find(".spinner").exists()).toBe(true);
    data.error.value = "No access";
    await wrapper.vm.$nextTick();
    expect(wrapper.find('[data-testid="widget-error"]').text()).toBe(
      "No access",
    );
  });

  it("refreshes on click and when the refresh key changes, and links to the query", async () => {
    const wrapper = mountWidget({ refreshKey: 0 });
    await wrapper.find('[data-testid="widget-refresh"]').trigger("click");
    await wrapper.setProps({ refreshKey: 1 });
    expect(data.run).toHaveBeenCalledTimes(2);
    const link = wrapper.findComponent(stubs.RouterLink);
    expect(link.props("to")).toEqual({
      name: "query",
      params: { owner: "o", db: "d" },
      query: {
        exec: ['select().ids("users")', "select().ids(1)"],
        context: ["select().aliases()"],
      },
    });
  });

  it("offers the edit actions in its menu", async () => {
    const wrapper = mountWidget({ first: true, last: false });
    const toggle = wrapper.find('[data-testid="widget-menu"]');
    expect(wrapper.find('[data-testid="widget-edit"]').exists()).toBe(false);
    await toggle.trigger("click");
    expect(toggle.attributes("aria-expanded")).toBe("true");
    expect(
      wrapper.find('[data-testid="widget-move-up"]').attributes("disabled"),
    ).toBeDefined();
    await wrapper.find('[data-testid="widget-move-up"]').trigger("click");
    await wrapper.find('[data-testid="widget-move-down"]').trigger("click");
    await wrapper.find('[data-testid="widget-size-4x3"]').trigger("click");
    expect(wrapper.emitted("move")).toEqual([[1]]);
    expect(wrapper.emitted("resize")).toEqual([[{ cols: 4, rows: 3 }]]);
    // Moves and sizes keep the menu open for more of them.
    await wrapper.find('[data-testid="widget-edit"]').trigger("click");
    expect(wrapper.emitted("edit")).toHaveLength(1);
    expect(wrapper.find('[data-testid="widget-edit"]').exists()).toBe(false);
    await toggle.trigger("click");
    await wrapper.find('[data-testid="widget-remove"]').trigger("click");
    expect(wrapper.emitted("remove")).toHaveLength(1);
    expect(wrapper.find('[data-testid="widget-remove"]').exists()).toBe(false);
  });

  it("closes the menu on Escape, with focus back on its button, and on a click outside", async () => {
    const wrapper = mount(DashboardWidget, {
      props: { widget, size: { cols: 2, rows: 1 } },
      global: { stubs },
      attachTo: document.body,
    });
    const toggle = wrapper.find('[data-testid="widget-menu"]');
    await toggle.trigger("click");
    await wrapper
      .find('[data-testid="widget-size-1x1"]')
      .trigger("keydown", { key: "Escape" });
    expect(wrapper.find(".menu-items").exists()).toBe(false);
    expect(document.activeElement).toBe(toggle.element);
    await toggle.trigger("click");
    // The outside click handler skips clicks in the same tick as the last one.
    await new Promise((resolve) => setTimeout(resolve));
    document.body.dispatchEvent(new Event("pointerdown", { bubbles: true }));
    document.body.click();
    await wrapper.vm.$nextTick();
    expect(wrapper.find(".menu-items").exists()).toBe(false);
    wrapper.unmount();
  });

  it("names a widget without a title", () => {
    const wrapper = mountWidget({ widget: { ...widget, title: "" } });
    expect(wrapper.find("h3").text()).toBe("Untitled");
  });

  it("moves earlier when not first", async () => {
    const wrapper = mountWidget({ first: false, last: true });
    await wrapper.find('[data-testid="widget-menu"]').trigger("click");
    await wrapper.find('[data-testid="widget-move-up"]').trigger("click");
    expect(wrapper.emitted("move")).toEqual([[-1]]);
  });

  it("offers to save or reset a changed view", async () => {
    const wrapper = mountWidget({});
    const note = () => wrapper.find('[data-testid="widget-view-changed"]');
    expect(note().exists()).toBe(false);
    data.displayChanged.value = true;
    await wrapper.vm.$nextTick();
    expect(note().text()).toContain("Unsaved view");
    const shown = { ...widget, view: "table" as const };
    data.shownWidget.mockReturnValue(shown);
    await wrapper.find('[data-testid="widget-view-save"]').trigger("click");
    expect(wrapper.emitted("update")).toEqual([[shown]]);
    await wrapper.find('[data-testid="widget-view-reset"]').trigger("click");
    expect(data.resetDisplay).toHaveBeenCalled();
  });

  it("shows a read-only widget without the actions that change it", async () => {
    data.displayChanged.value = true;
    const wrapper = mountWidget({ readOnly: true });
    await wrapper.vm.$nextTick();
    expect(wrapper.find('[data-testid="widget-menu"]').exists()).toBe(false);
    expect(wrapper.find('[data-testid="widget-view-save"]').exists()).toBe(
      false,
    );
    // The view can still be reset, it changes nothing saved.
    expect(wrapper.find('[data-testid="widget-view-reset"]').exists()).toBe(
      true,
    );
    expect(wrapper.find('[data-testid="widget-refresh"]').exists()).toBe(true);
  });

  it("links to the admin query editor in the admin view", () => {
    const { isAdmin, isAdminView } = useAdmin();
    (isAdmin as Ref<boolean>).value = true;
    (isAdminView as Ref<boolean>).value = true;
    const link = mountWidget({}).findComponent(stubs.RouterLink);
    expect(link.props("to")).toMatchObject({ name: "admin-query" });
  });
});
