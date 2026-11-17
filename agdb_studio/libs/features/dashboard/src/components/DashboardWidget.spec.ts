import { beforeEach, describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { ref } from "vue";

const { data, observer } = vi.hoisted(() => ({
  data: {
    error: undefined as unknown as import("vue").Ref<string | undefined>,
    loading: undefined as unknown as import("vue").Ref<boolean>,
    revision: undefined as unknown as import("vue").Ref<number>,
    query: undefined as unknown as import("vue").Ref<
      { lastRun?: Date } | undefined
    >,
    run: vi.fn(),
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

  it("emits edit actions in edit mode", async () => {
    const wrapper = mountWidget({ editing: true, first: true, last: false });
    expect(
      wrapper.find('[data-testid="widget-move-up"]').attributes("disabled"),
    ).toBeDefined();
    await wrapper.find('[data-testid="widget-move-up"]').trigger("click");
    await wrapper.find('[data-testid="widget-move-down"]').trigger("click");
    await wrapper.find('[data-testid="widget-edit"]').trigger("click");
    await wrapper.find('[data-testid="widget-remove"]').trigger("click");
    await wrapper.find('[data-testid="widget-size"]').trigger("click");
    await wrapper.find('[data-testid="widget-size-4x3"]').trigger("click");
    expect(wrapper.emitted("move")).toEqual([[1]]);
    expect(wrapper.emitted("resize")).toEqual([[{ cols: 4, rows: 3 }]]);
    expect(wrapper.emitted("edit")).toHaveLength(1);
    expect(wrapper.emitted("remove")).toHaveLength(1);
    expect(wrapper.find('[data-testid="widget-refresh"]').exists()).toBe(false);
  });

  it("moves earlier when not first", async () => {
    const wrapper = mountWidget({ editing: true, first: false, last: true });
    await wrapper.find('[data-testid="widget-move-up"]').trigger("click");
    expect(wrapper.emitted("move")).toEqual([[-1]]);
  });
});
