import { describe, it, expect, beforeEach, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { ref } from "vue";
import QueryRow from "./QueryRow.vue";
import { useQueryStore } from "../../composables/queryStore";
import type { QueryRow as QueryRowModel } from "../../composables/types";

describe("QueryRow", () => {
  const queryStore = useQueryStore();

  beforeEach(() => {
    queryStore.clearQueries();
  });

  const mountRow = (
    row: QueryRowModel,
    {
      queryId = "q1" as string | undefined,
      index = 0,
      resultIndex = undefined as number | undefined,
      showIndex = false,
      canMove = false,
      canRemove = false,
      canAdd = false,
      isRunning = false,
      attachTo = undefined as HTMLElement | undefined,
    } = {},
  ) =>
    mount(QueryRow, {
      attachTo,
      props: {
        row,
        index,
        resultIndex,
        showIndex,
        canMove,
        canRemove,
        canAdd,
        isRunning,
      },
      global: {
        provide: {
          queryId: ref(queryId),
          activeTab: ref("exec"),
        },
      },
    });

  it("adds a step to its own row", async () => {
    const query = queryStore.addQuery({ id: "q1" });
    const second = queryStore.addQueryRow("q1", "exec");
    const wrapper = mountRow(second!);

    await wrapper
      .findComponent({ name: "QueryStepInput" })
      .vm.$emit("confirm-step", "select");

    expect(query.value.rows.exec[0]?.steps).toHaveLength(0);
    expect(query.value.rows.exec[1]?.steps[0]?.type).toBe("select");
  });

  it("clears only its own steps", async () => {
    const query = queryStore.addQuery({ id: "q1" });
    const first = query.value.rows.exec[0]!;
    const second = queryStore.addQueryRow("q1", "exec")!;
    queryStore.addQueryStep("q1", "exec", first.id, {
      id: "a",
      type: "select",
    });
    queryStore.addQueryStep("q1", "exec", second.id, {
      id: "b",
      type: "select",
    });

    const wrapper = mountRow(query.value.rows.exec[1]!);
    await wrapper.find(".remove-button").trigger("click");

    expect(query.value.rows.exec[0]?.steps).toHaveLength(1);
    expect(query.value.rows.exec[1]?.steps).toHaveLength(0);
  });

  it("removes the row when allowed", async () => {
    const query = queryStore.addQuery({ id: "q1" });
    const second = queryStore.addQueryRow("q1", "exec")!;

    const wrapper = mountRow(second, { canRemove: true });
    await wrapper.find(".remove-row-button").trigger("click");

    expect(query.value.rows.exec.map((r) => r.id)).not.toContain(second.id);
  });

  it("shows a spinner and hides controls while running", () => {
    const query = queryStore.addQuery({ id: "q1" });
    const row = query.value.rows.exec[0]!;
    queryStore.addQueryStep("q1", "exec", row.id, { id: "a", type: "select" });

    const wrapper = mountRow(row, { canRemove: true, isRunning: true });

    expect(wrapper.find(".spinner-icon").exists()).toBe(true);
    expect(wrapper.find(".remove-button").exists()).toBe(false);
    expect(wrapper.find(".remove-row-button").exists()).toBe(false);
  });

  it("shows the row index only when asked to", () => {
    const row = { id: "r1", steps: [] };

    expect(mountRow(row).find(".row-index").exists()).toBe(false);
    const numbered = mountRow(row, {
      index: 2,
      resultIndex: 1,
      showIndex: true,
    }).find(".row-index");
    expect(numbered.text()).toBe("1");
    expect(numbered.attributes("title")).toBe("Result index");

    const skipped = mountRow(row, { index: 2, showIndex: true }).find(
      ".row-index",
    );
    expect(skipped.text()).toBe("");
    expect(skipped.attributes("title")).toBe("Empty queries are skipped");
  });

  it("emits add-row from the add button, disabled while running", async () => {
    const row = { id: "r1", steps: [] };

    expect(mountRow(row).find(".add-row-button").exists()).toBe(false);

    const wrapper = mountRow(row, { canAdd: true });
    await wrapper.find(".add-row-button").trigger("click");
    expect(wrapper.emitted("add-row")).toHaveLength(1);

    const running = mountRow(row, { canAdd: true, isRunning: true });
    expect(
      running.find(".add-row-button").attributes("disabled"),
    ).toBeDefined();
  });

  it("shows the drag handle only when the row can move", () => {
    const row = { id: "r1", steps: [] };

    expect(mountRow(row).find(".drag-handle").exists()).toBe(false);
    expect(mountRow(row, { canMove: true }).find(".drag-handle").exists()).toBe(
      true,
    );
  });

  it("starts a move drag with the whole row as the drag image", async () => {
    const wrapper = mountRow(
      { id: "r1", steps: [] },
      { index: 1, canMove: true },
    );
    const dataTransfer = {
      effectAllowed: "",
      setData: vi.fn(),
      setDragImage: vi.fn(),
    };

    await wrapper.find(".drag-handle").trigger("dragstart", { dataTransfer });

    expect(dataTransfer.effectAllowed).toBe("move");
    expect(dataTransfer.setData).toHaveBeenCalledWith("text/plain", "1");
    expect(dataTransfer.setDragImage).toHaveBeenCalledWith(
      wrapper.element,
      0,
      0,
    );
    expect(wrapper.emitted("drag-start")).toHaveLength(1);
  });

  it("moves with the arrow keys on the handle and ignores other keys", async () => {
    const wrapper = mountRow(
      { id: "r1", steps: [] },
      { index: 1, canMove: true },
    );
    const handle = wrapper.find(".drag-handle");

    await handle.trigger("keydown", { key: "ArrowUp" });
    await handle.trigger("keydown", { key: "ArrowDown" });
    await handle.trigger("keydown", { key: "Enter" });

    expect(wrapper.emitted("move")).toEqual([[0], [2]]);
  });

  it("places the clear button after the step input", () => {
    const query = queryStore.addQuery({ id: "q1" });
    const row = query.value.rows.exec[0]!;
    queryStore.addQueryStep("q1", "exec", row.id, { id: "a", type: "select" });

    const children = [...mountRow(row).find(".row-steps").element.children];
    const inputIndex = children.findIndex((el) =>
      el.classList.contains("query-step-input"),
    );
    expect(children[inputIndex + 1]?.classList).toContain("remove-button");
  });

  it("focuses the step input when the empty part of the row is clicked", async () => {
    queryStore.addQuery({ id: "q1" });
    const wrapper = mountRow(
      { id: "r1", steps: [] },
      { attachTo: document.body },
    );

    await wrapper.find(".row-steps").trigger("click");
    expect(document.activeElement).toBe(wrapper.find(".step-input").element);
    wrapper.unmount();
  });

  it("does nothing without a queryId", () => {
    const wrapper = mountRow({ id: "r1", steps: [] }, { queryId: undefined });
    const vm = wrapper.vm as unknown as {
      clearSteps: () => void;
      removeRow: () => void;
    };
    expect(() => {
      vm.clearSteps();
      vm.removeRow();
    }).not.toThrow();
  });
});
