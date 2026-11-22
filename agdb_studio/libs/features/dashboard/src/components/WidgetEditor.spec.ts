import { beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent, type Ref } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  return await importOriginal<typeof import("@agnesoft/agdb_api")>();
});

const { databases, seen, executeDbQueries } = vi.hoisted(() => ({
  executeDbQueries: vi.fn(),
  databases: { value: [] as { owner: string; db: string }[] },
  seen: {
    queryId: undefined as string | undefined,
    tabs: undefined as unknown,
    keepView: undefined as unknown,
    route: undefined as undefined | { params: Record<string, unknown> },
  },
}));

vi.mock(
  "@agdb-studio/query/src/composables/dbExec",
  async (importOriginal) => ({
    ...(await importOriginal<
      typeof import("@agdb-studio/query/src/composables/dbExec")
    >()),
    executeDbQueries,
  }),
);
vi.mock("@agdb-studio/db/src/composables/dbStore", () => ({
  useDbStore: () => ({ databases }),
}));
vi.mock("@agdb-studio/query/src/components/view/QueryView.vue", async () => {
  const { useMinimalRoute } =
    await import("@agdb-studio/query/src/composables/dbExec");
  return {
    default: defineComponent({
      name: "QueryView",
      props: {
        queryId: { type: String, default: undefined },
        tabs: { type: Array, default: undefined },
        keepView: { type: Boolean, default: false },
      },
      setup(props) {
        seen.queryId = props.queryId;
        seen.keepView = props.keepView;
        seen.tabs = props.tabs;
        seen.route = useMinimalRoute();
        return () => null;
      },
    }),
  };
});

import WidgetEditor from "./WidgetEditor.vue";
import { useQueryStore } from "@agdb-studio/query/src/composables/queryStore";
import { createWidget } from "../utils/dashboardDocument";
import { toQueryRows } from "../utils/widgetRows";
import type { Widget } from "../types";
import { useAdmin } from "@agdb-studio/router/src/admin";

const USERS = 'select().ids("users")';
const queryStore = useQueryStore();

const mountEditor = (widget: Widget, header?: string) =>
  mount(WidgetEditor, { props: { widget, header } });

const button = (wrapper: ReturnType<typeof mountEditor>, id: string) =>
  wrapper.find(`[data-testid="${id}"]`);

describe("WidgetEditor", () => {
  beforeEach(() => {
    queryStore.clearQueries();
    databases.value = [
      { owner: "a", db: "one" },
      { owner: "b", db: "two" },
    ];
    seen.route = undefined;
    executeDbQueries.mockReset();
    executeDbQueries.mockResolvedValue([{ result: 1, elements: [] }]);
  });

  it("starts a new widget on the first database and needs a query", async () => {
    const wrapper = mountEditor(
      createWidget({ owner: "", db: "" }),
      "Add widget",
    );
    expect(wrapper.find("h2").text()).toBe("Add widget");
    expect(wrapper.attributes("aria-label")).toBe("Add widget");
    expect(
      (button(wrapper, "widget-database").element as HTMLSelectElement).value,
    ).toBe("a/one");
    expect(seen.tabs).toEqual(["exec", "context"]);
    // Pins of elements missing from the preview stay, as on the dashboard.
    expect(seen.keepView).toBe(true);
    expect(seen.route?.params).toEqual({ owner: "a", db: "one" });
    expect(
      button(wrapper, "widget-editor-done").attributes("disabled"),
    ).toBeDefined();
    expect(button(wrapper, "widget-editor-hint").text()).toContain(
      "Build a query",
    );

    // The builder works on the editor's entry; a query enables Done.
    queryStore.getQuery(seen.queryId!)!.value.rows.exec = toQueryRows([
      { id: "r1", query: USERS },
    ]);
    queryStore.getQuery(seen.queryId!)!.value.rows.context = toQueryRows([
      { id: "c1", query: "select()" },
    ]);
    await wrapper.vm.$nextTick();
    // An unfinished row would save a widget that cannot run.
    expect(button(wrapper, "widget-editor-hint").text()).toContain(
      "cannot run yet",
    );
    expect(
      button(wrapper, "widget-editor-done").attributes("disabled"),
    ).toBeDefined();
    queryStore.getQuery(seen.queryId!)!.value.rows.context = [];
    await wrapper.vm.$nextTick();
    expect(button(wrapper, "widget-editor-hint").text()).not.toContain("Build");
    await button(wrapper, "widget-database").setValue("b/two");
    expect(seen.route?.params).toEqual({ owner: "b", db: "two" });
    await button(wrapper, "widget-title").setValue("  ");
    await button(wrapper, "widget-refresh-seconds").setValue("30");
    await button(wrapper, "widget-editor-done").trigger("click");
    const [widget] = wrapper.emitted("done")![0] as [Widget];
    expect(widget).toMatchObject({
      title: "Untitled",
      owner: "b",
      db: "two",
      refreshSeconds: 30,
      rows: { exec: [{ id: "r1", query: USERS }], context: [] },
    });
    // Size and place are set on the dashboard grid, not here.
    expect(widget).not.toHaveProperty("size");

    wrapper.unmount();
    expect(queryStore.getQuery(seen.queryId!)).toBeUndefined();
  });

  it("edits a widget, keeping a database that is not listed", async () => {
    const wrapper = mountEditor(
      createWidget({
        owner: "x",
        db: "hidden",
        title: "Old",
        refreshSeconds: 60,
        rows: { exec: [{ id: "r1", query: USERS }], context: [] },
      }),
    );
    expect(wrapper.find("h2").text()).toBe("Edit widget");
    const options = button(wrapper, "widget-database")
      .findAll("option")
      .map((o) => o.text());
    expect(options).toEqual(["x/hidden", "a/one", "b/two"]);

    expect(
      button(wrapper, "widget-refresh-seconds")
        .findAll("option")
        .map((o) => o.text()),
    ).toEqual(["Off", "10 s", "30 s", "1 min", "5 min", "15 min", "1 h"]);
    expect(
      (button(wrapper, "widget-refresh-seconds").element as HTMLSelectElement)
        .value,
    ).toBe("60");
    await button(wrapper, "widget-refresh-seconds").setValue("0");
    await button(wrapper, "widget-editor-done").trigger("click");
    const [widget] = wrapper.emitted("done")![0] as [Widget];
    expect(widget.title).toBe("Old");
    expect(widget.refreshSeconds).toBeUndefined();
  });

  it("keeps a saved refresh interval that is not offered, and drops a broken one", () => {
    const options = (refreshSeconds: number) =>
      button(
        mountEditor(createWidget({ owner: "a", db: "one", refreshSeconds })),
        "widget-refresh-seconds",
      )
        .findAll("option")
        .map((o) => o.text());
    expect(options(45)).toContain("45 s");
    expect(options(5)).not.toContain("5 s");
  });

  it("previews with admin access in the admin view", async () => {
    const { isAdmin, isAdminView } = useAdmin();
    (isAdmin as Ref<boolean>).value = true;
    (isAdminView as Ref<boolean>).value = true;
    mountEditor(
      createWidget({
        owner: "a",
        db: "one",
        rows: { exec: [{ id: "r1", query: USERS }], context: [] },
      }),
    );
    await flushPromises();
    expect(executeDbQueries).toHaveBeenLastCalledWith(
      [expect.anything()],
      false,
      { params: { owner: "a", db: "one" }, name: "admin-query" },
      { quiet: true },
    );
  });

  it("previews the widget at once, and again after edits", async () => {
    vi.useFakeTimers();
    try {
      const wrapper = mountEditor(
        createWidget({
          owner: "a",
          db: "one",
          rows: { exec: [{ id: "r1", query: USERS }], context: [] },
        }),
      );
      await flushPromises();
      expect(executeDbQueries).toHaveBeenCalledTimes(1);
      expect(executeDbQueries).toHaveBeenLastCalledWith(
        [expect.anything()],
        false,
        { params: { owner: "a", db: "one" }, name: "query" },
        // Errors show in the hint, not as notifications.
        { quiet: true },
      );
      const query = () => queryStore.getQuery(seen.queryId!)!.value;
      expect(query().results.exec).toEqual([{ result: 1, elements: [] }]);

      await button(wrapper, "widget-database").setValue("b/two");
      await flushPromises();
      expect(executeDbQueries).toHaveBeenLastCalledWith(
        [expect.anything()],
        false,
        { params: { owner: "b", db: "two" }, name: "query" },
        // Errors show in the hint, not as notifications.
        { quiet: true },
      );

      // Builder edits run after a pause in typing, once the rows can run.
      executeDbQueries.mockRejectedValueOnce(new Error("boom"));
      query().rows.exec = toQueryRows([{ id: "r1", query: "select().ids(1)" }]);
      await flushPromises();
      expect(executeDbQueries).toHaveBeenCalledTimes(2);
      vi.advanceTimersByTime(600);
      await flushPromises();
      expect(executeDbQueries).toHaveBeenCalledTimes(3);
      const hint = button(wrapper, "widget-editor-hint");
      expect(hint.text()).toBe("Preview failed: boom");
      expect(hint.classes()).toContain("error");

      query().rows.exec = [];
      await flushPromises();
      vi.advanceTimersByTime(600);
      await flushPromises();
      expect(executeDbQueries).toHaveBeenCalledTimes(3);
    } finally {
      vi.useRealTimers();
    }
  });

  it("shows only the latest preview", async () => {
    let finishFirst: (value: unknown) => void = () => {};
    let failSecond: (error: unknown) => void = () => {};
    executeDbQueries
      .mockImplementationOnce(
        () => new Promise((resolve) => (finishFirst = resolve)),
      )
      .mockImplementationOnce(
        () => new Promise((_, reject) => (failSecond = reject)),
      );
    const wrapper = mountEditor(
      createWidget({
        owner: "a",
        db: "one",
        rows: { exec: [{ id: "r1", query: USERS }], context: [] },
      }),
    );
    await wrapper.vm.$nextTick();
    expect(button(wrapper, "widget-editor-hint").text()).toBe(
      "Updating preview…",
    );
    await button(wrapper, "widget-database").setValue("b/two");
    await button(wrapper, "widget-database").setValue("a/one");
    await flushPromises();
    finishFirst([{ result: 9, elements: [] }]);
    failSecond(new Error("old"));
    await flushPromises();
    expect(button(wrapper, "widget-editor-hint").text()).toBe("");
    expect(queryStore.getQuery(seen.queryId!)!.value.results.exec).toEqual([
      { result: 1, elements: [] },
    ]);
  });

  it("lets a run from the query view win over a running preview", async () => {
    let finishPreview: (value: unknown) => void = () => {};
    executeDbQueries.mockImplementationOnce(
      () => new Promise((resolve) => (finishPreview = resolve)),
    );
    const wrapper = mountEditor(
      createWidget({
        owner: "a",
        db: "one",
        rows: { exec: [{ id: "r1", query: USERS }], context: [] },
      }),
    );
    await flushPromises();
    const query = queryStore.getQuery(seen.queryId!)!.value;
    query.isRunning = true;
    await wrapper.vm.$nextTick();
    expect(button(wrapper, "widget-editor-hint").text()).toBe("");
    const manual = [{ result: 5, elements: [] }];
    query.results.exec = manual;
    query.isRunning = false;
    finishPreview([{ result: 9, elements: [] }]);
    await flushPromises();
    expect(query.results.exec).toEqual(manual);
  });

  it("clears results of the previous database when the preview cannot run", async () => {
    const wrapper = mountEditor(
      createWidget({
        owner: "a",
        db: "one",
        rows: { exec: [{ id: "r1", query: "select().ids(" }], context: [] },
      }),
    );
    await flushPromises();
    const query = queryStore.getQuery(seen.queryId!)!.value;
    query.results.exec = [{ result: 1, elements: [] }];
    await button(wrapper, "widget-database").setValue("b/two");
    expect(query.results.exec).toBeUndefined();
    expect(executeDbQueries).not.toHaveBeenCalled();
  });

  it("asks for a database when none is available, and cancels", async () => {
    databases.value = [];
    const wrapper = mountEditor(createWidget({ owner: "", db: "" }));
    expect(button(wrapper, "widget-editor-hint").text()).toContain(
      "Pick a database",
    );
    await button(wrapper, "widget-editor-done").trigger("click");
    expect(wrapper.emitted("done")).toBeUndefined();
    await button(wrapper, "widget-editor-cancel").trigger("click");
    await wrapper.trigger("keydown", { key: "Escape" });
    await button(wrapper, "widget-title").trigger("keydown", { key: "Escape" });
    expect(wrapper.emitted("cancel")).toHaveLength(3);
  });

  it("leaves Escape inside the query view to the query view", async () => {
    const wrapper = mount(WidgetEditor, {
      props: { widget: createWidget({ owner: "a", db: "one" }) },
      global: {
        stubs: { QueryView: { template: '<input data-testid="builder" />' } },
      },
    });
    await button(wrapper, "builder").trigger("keydown", { key: "Escape" });
    expect(wrapper.emitted("cancel")).toBeUndefined();
  });

  it("uses a fresh store entry per editor", () => {
    const first = mountEditor(createWidget({ owner: "a", db: "one" }));
    const id = seen.queryId;
    mountEditor(createWidget({ owner: "a", db: "one" }));
    expect(seen.queryId).not.toBe(id);
    first.unmount();
    expect(queryStore.getQuery(seen.queryId!)).toBeDefined();
  });
});
