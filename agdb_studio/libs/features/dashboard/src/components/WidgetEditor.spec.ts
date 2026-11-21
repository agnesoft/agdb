import { beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent } from "vue";
import { mount } from "@vue/test-utils";
vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  return await importOriginal<typeof import("@agnesoft/agdb_api")>();
});

const { databases, seen } = vi.hoisted(() => ({
  databases: { value: [] as { owner: string; db: string }[] },
  seen: {
    queryId: undefined as string | undefined,
    tabs: undefined as unknown,
    route: undefined as undefined | { params: Record<string, unknown> },
  },
}));

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
      },
      setup(props) {
        seen.queryId = props.queryId;
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
    expect(button(wrapper, "widget-editor-hint").text()).toContain("saved");
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

    // Too short, negative or too long would flood the server.
    for (const value of ["5", "-5", "86401"]) {
      await button(wrapper, "widget-refresh-seconds").setValue(value);
      expect(button(wrapper, "widget-editor-hint").text()).toContain(
        "0 (off) or 10 to 86400 seconds",
      );
      expect(
        button(wrapper, "widget-editor-done").attributes("disabled"),
      ).toBeDefined();
    }
    await button(wrapper, "widget-refresh-seconds").setValue("0");
    await button(wrapper, "widget-editor-done").trigger("click");
    const [widget] = wrapper.emitted("done")![0] as [Widget];
    expect(widget.title).toBe("Old");
    expect(widget.refreshSeconds).toBeUndefined();
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
