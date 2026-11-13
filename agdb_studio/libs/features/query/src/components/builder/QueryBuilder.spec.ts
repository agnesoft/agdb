import { describe, it, expect, beforeEach, vi } from "vitest";
import { mount } from "@vue/test-utils";
import QueryBuilder from "./QueryBuilder.vue";
import { useQueryStore } from "../../composables/queryStore";
import { ref } from "vue";

describe("QueryBuilder", () => {
  let queryStore: ReturnType<typeof useQueryStore>;

  beforeEach(() => {
    queryStore = useQueryStore();
    queryStore.clearQueries();
  });

  it("renders the query builder", () => {
    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref("test-query-1"),
        },
      },
    });
    expect(wrapper.find(".query-builder").exists()).toBe(true);
    expect(wrapper.find(".query-rows").exists()).toBe(true);
    expect(wrapper.find("button.button-primary").exists()).toBe(true);
  });

  it("renders with exec tab styling", () => {
    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref("test-query-1"),
          activeTab: ref("exec"),
        },
      },
    });
    expect(wrapper.find(".run-query-button").classes()).toContain(
      "button-primary",
    );
  });

  it("renders with exec_mut tab styling", () => {
    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref("test-query-1"),
          activeTab: ref("exec_mut"),
        },
      },
    });
    expect(wrapper.find(".run-query-button").classes()).toContain(
      "button-warning",
    );
  });

  it("renders with context tab styling", () => {
    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref("test-query-1"),
          activeTab: ref("context"),
        },
      },
    });
    expect(wrapper.find(".run-query-button").classes()).toContain(
      "button-primary",
    );
  });

  it("adds a new step when confirm-step is emitted", async () => {
    const queryId = "test-query-3";
    queryStore.addQuery({ id: queryId });

    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref(queryId),
        },
      },
    });

    const stepInput = wrapper.findComponent({ name: "QueryStepInput" });
    await stepInput.vm.$emit("confirm-step", "select");

    const query = queryStore.getQuery(queryId);
    expect(query?.value.rows.exec[0]?.steps.length).toBe(1);
    expect(query?.value.rows.exec[0]?.steps[0]?.type).toBe("select");
  });

  it("handles missing queryId gracefully", () => {
    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref(undefined),
        },
      },
    });
    expect(wrapper.find(".query-builder").exists()).toBe(true);
  });

  it("does nothing when addRow is called without a queryId", () => {
    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref(undefined),
        },
      },
    });

    expect(() => {
      (wrapper.vm as unknown as { addRow: () => void }).addRow();
    }).not.toThrow();
    expect(wrapper.find(".add-row-button").exists()).toBe(false);
  });

  it("shows stop label, spinner and keeps the run button enabled while running", () => {
    const queryId = "test-query-running";
    const query = queryStore.addQuery({ id: queryId });
    query.value.isRunning = true;

    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref(queryId),
        },
      },
    });

    const button = wrapper.find(".run-query-button");
    expect(button.text()).toContain("Stop query");
    expect(button.attributes("disabled")).toBeUndefined();
    expect(wrapper.find(".spinner-icon").exists()).toBe(true);
  });

  it("runs the query when the run button is clicked", async () => {
    const queryId = "test-query-run-click";
    const query = queryStore.addQuery({ id: queryId });
    const rowId = query.value.rows.exec[0]?.id ?? "";
    queryStore.addQueryStep(queryId, "exec", rowId, {
      id: "s1",
      type: "select",
    });
    queryStore.addQueryStep(queryId, "exec", rowId, {
      id: "s2",
      type: "select.ids",
    });

    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref(queryId),
          activeTab: ref("exec"),
        },
      },
    });

    await wrapper.find(".run-query-button").trigger("click");

    expect(query.value.lastRun).toBeInstanceOf(Date);
    expect(query.value.isRunning).toBe(false);
  });

  it("shows QueryStepInput component", () => {
    queryStore.addQuery({ id: "test-query-4" });
    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref("test-query-4"),
        },
      },
    });
    const stepInput = wrapper.findComponent({ name: "QueryStepInput" });
    expect(stepInput.exists()).toBe(true);
  });

  it("hides clear button while query is running", async () => {
    const queryId = "test-query-clear-disabled";
    const query = queryStore.addQuery({ id: queryId });
    queryStore.addQueryStep(
      queryId,
      "exec",
      query.value.rows.exec[0]?.id ?? "",
      {
        id: "s1",
        type: "select",
      },
    );
    query.value.isRunning = true;

    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref(queryId),
          activeTab: ref("exec"),
        },
      },
    });

    const removeButton = wrapper.find(".remove-button");
    expect(removeButton.exists()).toBe(false);
  });

  it("can remove all steps", async () => {
    const queryId = "test-query-5";
    queryStore.addQuery({ id: queryId });

    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref(queryId),
        },
      },
    });

    const stepInput = wrapper.findComponent({ name: "QueryStepInput" });
    await stepInput.vm.$emit("confirm-step", "select");
    await stepInput.vm.$emit("confirm-step", "search");

    let query = queryStore.getQuery(queryId);
    expect(query?.value.rows.exec[0]?.steps.length).toBe(2);

    const removeButton = wrapper.find(".remove-button");
    await removeButton.trigger("click");

    query = queryStore.getQuery(queryId);
    expect(query?.value.rows.exec[0]?.steps.length).toBe(0);
  });

  it("adds and removes query rows", async () => {
    const queryId = "test-query-rows";
    queryStore.addQuery({ id: queryId });

    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref(queryId),
          activeTab: ref("exec"),
        },
      },
    });

    // A single row cannot be removed.
    expect(wrapper.findAllComponents({ name: "QueryRow" })).toHaveLength(1);
    expect(wrapper.find(".remove-row-button").exists()).toBe(false);

    await wrapper.find(".add-row-button").trigger("click");
    const rows = wrapper.findAllComponents({ name: "QueryRow" });
    expect(rows).toHaveLength(2);
    // Both rows are empty, so neither gets a result index yet.
    expect(wrapper.findAll(".row-index").map((el) => el.text())).toEqual([
      "",
      "",
    ]);

    await wrapper.find(".remove-row-button").trigger("click");
    expect(wrapper.findAllComponents({ name: "QueryRow" })).toHaveLength(1);
    expect(queryStore.getQuery(queryId)?.value.rows.exec).toHaveLength(1);
  });

  it("disables adding rows while running", () => {
    const queryId = "test-query-rows-running";
    const query = queryStore.addQuery({ id: queryId });
    query.value.isRunning = true;

    const wrapper = mount(QueryBuilder, {
      global: { provide: { queryId: ref(queryId) } },
    });

    expect(
      wrapper.find(".add-row-button").attributes("disabled"),
    ).toBeDefined();
  });

  const mountWithRows = (queryId: string, rowCount: number) => {
    const query = queryStore.addQuery({ id: queryId });
    for (let i = 1; i < rowCount; i++) queryStore.addQueryRow(queryId, "exec");
    const wrapper = mount(QueryBuilder, {
      global: {
        provide: { queryId: ref(queryId), activeTab: ref("exec") },
      },
    });
    const rowIds = () => query.value.rows.exec.map((r) => r.id);
    return { query, wrapper, rowIds };
  };

  it("hides and shows the queries with a summary when there are several", async () => {
    const { query, wrapper } = mountWithRows("test-hide", 1);
    expect(wrapper.find(".toggle-rows-button").exists()).toBe(false);

    queryStore.addQueryRow("test-hide", "exec");
    const [first, second] = query.value.rows.exec;
    queryStore.addQueryStep("test-hide", "exec", first!.id, {
      id: "a",
      type: "select",
    });
    queryStore.addQueryStep("test-hide", "exec", second!.id, {
      id: "b",
      type: "search",
    });
    await wrapper.vm.$nextTick();

    const toggle = wrapper.find(".toggle-rows-button");
    expect(toggle.text()).toContain("Hide queries");
    await toggle.trigger("click");

    expect(wrapper.findAllComponents({ name: "QueryRow" })).toHaveLength(0);
    expect(wrapper.find(".hidden-count").text()).toBe("2 queries");
    expect(wrapper.find(".hidden-preview").text()).toBe("select | search");
    expect(wrapper.find(".hidden-summary").classes()).not.toContain("invalid");
    expect(toggle.attributes("aria-expanded")).toBe("false");

    await wrapper.find(".hidden-summary").trigger("click");
    expect(wrapper.findAllComponents({ name: "QueryRow" })).toHaveLength(2);
  });

  it("flags invalid steps in the hidden summary and always shows a single row", async () => {
    const { query, wrapper } = mountWithRows("test-hide-invalid", 2);
    queryStore.addQueryStep(
      "test-hide-invalid",
      "exec",
      query.value.rows.exec[0]!.id,
      {
        id: "a",
        type: "insert",
      },
    );
    await wrapper.find(".toggle-rows-button").trigger("click");
    expect(wrapper.find(".hidden-summary").classes()).toContain("invalid");

    queryStore.deleteQueryRow(
      "test-hide-invalid",
      "exec",
      query.value.rows.exec[1]!.id,
    );
    await wrapper.vm.$nextTick();
    expect(wrapper.findAllComponents({ name: "QueryRow" })).toHaveLength(1);
  });

  it("moves rows with the keyboard", async () => {
    const { wrapper, rowIds } = mountWithRows("test-move-keys", 3);
    const ids = rowIds();

    await wrapper.findAll(".drag-handle")[0]!.trigger("keydown", {
      key: "ArrowDown",
    });
    expect(rowIds()).toEqual([ids[1], ids[0], ids[2]]);
  });

  it("reorders rows by dragging and dropping before or after a row", async () => {
    const { wrapper, rowIds } = mountWithRows("test-drag", 3);
    const ids = rowIds();
    const dataTransfer = { setData: vi.fn(), setDragImage: vi.fn() };
    const rowAt = (i: number) => wrapper.findAll(".query-input")[i]!;

    // Drag row 0 onto the lower half of row 2 -> it goes last.
    await wrapper.findAll(".drag-handle")[0]!.trigger("dragstart", {
      dataTransfer,
    });
    expect(rowAt(0).classes()).toContain("dragging");
    await rowAt(2).trigger("dragover", { clientY: 10 });
    expect(rowAt(2).classes()).toContain("drop-after");
    await rowAt(2).trigger("drop");
    expect(rowIds()).toEqual([ids[1], ids[2], ids[0]]);
    expect(wrapper.find(".dragging").exists()).toBe(false);

    // Drag the last row onto the upper half of row 0 -> it goes first.
    await wrapper.findAll(".drag-handle")[2]!.trigger("dragstart", {
      dataTransfer,
    });
    await rowAt(0).trigger("dragover", { clientY: -1 });
    expect(rowAt(0).classes()).toContain("drop-before");
    await rowAt(0).trigger("drop");
    expect(rowIds()).toEqual(ids);
  });

  it("ignores drags that did not start from a row handle", async () => {
    const { wrapper, rowIds } = mountWithRows("test-drag-foreign", 2);
    const ids = rowIds();
    const row = wrapper.findAll(".query-input")[1]!;

    await row.trigger("dragover", { clientY: 10 });
    expect(row.classes()).not.toContain("drop-after");
    await row.trigger("drop");
    await row.trigger("dragend");
    expect(rowIds()).toEqual(ids);
  });

  it("numbers only non-empty rows, matching the server's result indexes", async () => {
    const { query, wrapper } = mountWithRows("test-result-index", 3);
    const [first, , third] = query.value.rows.exec;
    queryStore.addQueryStep("test-result-index", "exec", first!.id, {
      id: "a",
      type: "select",
    });
    queryStore.addQueryStep("test-result-index", "exec", third!.id, {
      id: "b",
      type: "search",
    });
    await wrapper.vm.$nextTick();

    expect(wrapper.findAll(".row-index").map((el) => el.text())).toEqual([
      "0",
      "",
      "1",
    ]);
  });

  it("counts only non-empty rows in the hidden summary", async () => {
    const { query, wrapper } = mountWithRows("test-hidden-count", 3);
    queryStore.addQueryStep(
      "test-hidden-count",
      "exec",
      query.value.rows.exec[0]!.id,
      { id: "a", type: "select" },
    );
    await wrapper.vm.$nextTick();
    await wrapper.find(".toggle-rows-button").trigger("click");

    expect(wrapper.find(".hidden-count").text()).toBe("1 query");
    expect(wrapper.find(".hidden-preview").text()).toBe("select");
  });

  it("remembers hidden rows per tab", async () => {
    const activeTab = ref<"exec" | "exec_mut">("exec");
    queryStore.addQuery({ id: "test-hide-tabs" });
    queryStore.addQueryRow("test-hide-tabs", "exec");
    queryStore.addQueryRow("test-hide-tabs", "exec_mut");
    const wrapper = mount(QueryBuilder, {
      global: { provide: { queryId: ref("test-hide-tabs"), activeTab } },
    });

    await wrapper.find(".toggle-rows-button").trigger("click");
    expect(wrapper.find(".hidden-summary").exists()).toBe(true);

    activeTab.value = "exec_mut";
    await wrapper.vm.$nextTick();
    expect(wrapper.find(".hidden-summary").exists()).toBe(false);

    activeTab.value = "exec";
    await wrapper.vm.$nextTick();
    expect(wrapper.find(".hidden-summary").exists()).toBe(true);
  });

  it("measures the drop position from the row, not the element under the pointer", async () => {
    const { wrapper, rowIds } = mountWithRows("test-drag-rect", 2);
    const ids = rowIds();
    const target = wrapper.findAll(".query-input")[0]!;
    // A wrapped row 60px tall; the pointer is on its second line, over a child element.
    target.element.getBoundingClientRect = () =>
      ({ top: 100, height: 60 }) as DOMRect;

    await wrapper.findAll(".drag-handle")[1]!.trigger("dragstart", {
      dataTransfer: { setData: vi.fn(), setDragImage: vi.fn() },
    });
    await target.trigger("dragover", { clientY: 150 });
    expect(target.classes()).toContain("drop-after");

    await target.trigger("drop");
    expect(rowIds()).toEqual(ids);
  });

  it("clears the drop indicator when the pointer leaves the rows", async () => {
    const { wrapper } = mountWithRows("test-drag-leave", 2);
    const rows = wrapper.find(".query-rows");
    const target = wrapper.findAll(".query-input")[1]!;

    await wrapper.findAll(".drag-handle")[0]!.trigger("dragstart", {
      dataTransfer: { setData: vi.fn(), setDragImage: vi.fn() },
    });
    await target.trigger("dragover", { clientY: 10 });
    expect(target.classes()).toContain("drop-after");

    // Moving between rows keeps it.
    await rows.trigger("dragleave", {
      relatedTarget: wrapper.findAll(".query-input")[0]!.element,
    });
    expect(target.classes()).toContain("drop-after");

    // Leaving the rows clears it.
    await rows.trigger("dragleave", { relatedTarget: document.body });
    expect(target.classes()).not.toContain("drop-after");
  });
});
