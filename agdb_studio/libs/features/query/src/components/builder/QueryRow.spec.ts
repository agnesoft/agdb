import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";

const { addNotification } = vi.hoisted(() => ({ addNotification: vi.fn() }));
vi.mock("@agdb-studio/notification/src/composables/notificationStore", () => ({
  addNotification,
}));

import { ref } from "vue";
import QueryRow from "./QueryRow.vue";
import { useQueryStore } from "../../composables/queryStore";
import type { QueryRow as QueryRowModel } from "../../composables/types";
import { formatSteps, resetCopiedSteps } from "../../utils/queryRowClipboard";

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

  describe("copy and paste", () => {
    let clipboard = "";
    const stubClipboard = (mode: "ok" | "none" | "denied" = "ok") =>
      vi.stubGlobal("navigator", {
        clipboard: {
          ok: {
            writeText: vi.fn(async (text: string) => {
              clipboard = text;
            }),
            readText: vi.fn(async () => clipboard),
          },
          none: undefined,
          denied: {
            writeText: vi.fn().mockRejectedValue(new Error("denied")),
            readText: vi.fn().mockRejectedValue(new Error("denied")),
          },
        }[mode],
      });

    const pasteEvent = (text: string) => {
      const event = new Event("paste", { bubbles: true, cancelable: true });
      Object.defineProperty(event, "clipboardData", {
        value: { getData: () => text },
      });
      return event;
    };

    beforeEach(() => {
      clipboard = "";
      resetCopiedSteps();
      addNotification.mockClear();
      stubClipboard();
    });
    afterEach(() => vi.unstubAllGlobals());

    const setup = () => {
      const query = queryStore.addQuery({ id: "q1" });
      const first = query.value.rows.exec[0]!;
      queryStore.addQueryStep("q1", "exec", first.id, {
        id: "s1",
        type: "select",
      });
      queryStore.addQueryStep("q1", "exec", first.id, {
        id: "s2",
        type: "select.ids",
        args: [[{ selectedOption: "signed", value: "1" }]],
      });
      return { query, first };
    };

    it("copies a row and pastes it below a row with steps", async () => {
      const { query, first } = setup();
      const wrapper = mountRow(first);
      await wrapper.find(".copy-row-button").trigger("click");
      await flushPromises();
      expect(clipboard).toBe("QueryBuilder::select().ids(1).query()");
      expect(addNotification).toHaveBeenCalledWith(
        expect.objectContaining({ type: "success" }),
      );

      expect(wrapper.find(".paste-row-button").attributes("title")).toContain(
        "below",
      );
      await wrapper.find(".paste-row-button").trigger("click");
      await flushPromises();
      const rows = query.value.rows.exec;
      expect(rows).toHaveLength(2);
      expect(rows[1]?.steps.map((s) => s.type)).toEqual([
        "select",
        "select.ids",
      ]);
      expect(rows[1]?.steps[0]?.id).not.toBe("s1");
    });

    it("pastes into an empty row, with no copy button there", async () => {
      const { query, first } = setup();
      const second = queryStore.addQueryRow("q1", "exec")!;
      clipboard = formatSteps(first.steps);
      const wrapper = mountRow(second);
      expect(wrapper.find(".copy-row-button").exists()).toBe(false);
      expect(wrapper.find(".paste-row-button").attributes("title")).toContain(
        "here",
      );
      await wrapper.find(".paste-row-button").trigger("click");
      await flushPromises();
      expect(query.value.rows.exec).toHaveLength(2);
      expect(query.value.rows.exec[1]?.steps).toHaveLength(2);
    });

    it("explains why the clipboard text cannot be pasted", async () => {
      const { query, first } = setup();
      clipboard = "select().nope()";
      const wrapper = mountRow(first);
      await wrapper.find(".paste-row-button").trigger("click");
      await flushPromises();
      expect(query.value.rows.exec).toHaveLength(1);
      expect(addNotification).toHaveBeenCalledWith({
        type: "warning",
        title: "Could not paste the query",
        message: "nope() cannot follow select()",
      });
    });

    it("pastes Rust code copied from elsewhere", async () => {
      const { query, first } = setup();
      clipboard =
        'let r = db.exec(&QueryBuilder::search().from("users").where_().node().query())?;';
      const wrapper = mountRow(first);
      await wrapper.find(".paste-row-button").trigger("click");
      await flushPromises();
      expect(query.value.rows.exec[1]?.steps.map((s) => s.type)).toEqual([
        "search",
        "search.from",
        "search.where",
        "search.where.node",
      ]);
    });

    it("keeps a local copy when the browser has no clipboard API", async () => {
      stubClipboard("none");
      const { query, first } = setup();
      const wrapper = mountRow(first);
      await wrapper.find(".copy-row-button").trigger("click");
      await flushPromises();
      expect(addNotification).toHaveBeenCalledWith(
        expect.objectContaining({ type: "info" }),
      );
      await wrapper.find(".paste-row-button").trigger("click");
      await flushPromises();
      expect(query.value.rows.exec).toHaveLength(2);
    });

    it("reports denied clipboard access and pastes nothing", async () => {
      stubClipboard("denied");
      const { query, first } = setup();
      const wrapper = mountRow(first);
      await wrapper.find(".copy-row-button").trigger("click");
      await flushPromises();
      expect(addNotification).toHaveBeenCalledWith(
        expect.objectContaining({ type: "error" }),
      );
      await wrapper.find(".paste-row-button").trigger("click");
      await flushPromises();
      expect(addNotification).toHaveBeenCalledWith(
        expect.objectContaining({ title: "Clipboard access denied" }),
      );
      expect(query.value.rows.exec).toHaveLength(1);
    });

    it("validates pasted steps for the active tab", async () => {
      const query = queryStore.addQuery({ id: "q1" });
      const row = query.value.rows.exec_mut[0]!;
      // A read query pasted into the mutation tab.
      clipboard = "select().node_count()";
      const wrapper = mount(QueryRow, {
        props: {
          row,
          index: 0,
          showIndex: false,
          canMove: false,
          canRemove: false,
          canAdd: false,
          isRunning: false,
        },
        global: { provide: { queryId: ref("q1"), activeTab: ref("exec_mut") } },
      });
      await wrapper.find(".paste-row-button").trigger("click");
      await flushPromises();
      expect(query.value.rows.exec_mut[0]?.steps.map((s) => s.invalid)).toEqual(
        [true, true],
      );
      expect(query.value.rows.exec[0]?.steps).toHaveLength(0);
    });

    it("pastes query code with Ctrl+V", () => {
      const { query, first } = setup();
      const wrapper = mountRow(first);
      const row = pasteEvent(formatSteps(first.steps));
      wrapper.find("[contenteditable]").element.dispatchEvent(row);
      expect(row.defaultPrevented).toBe(true);
      expect(query.value.rows.exec).toHaveLength(2);
    });

    it.each([
      ["a value list", "1, 2"],
      ["a single word", "select"],
      ["text that is not a query", "not.a.query"],
    ])("leaves %s to the step input", (_, text) => {
      const { query, first } = setup();
      const wrapper = mountRow(first);
      const event = pasteEvent(text);
      wrapper.find("[contenteditable]").element.dispatchEvent(event);
      expect(event.defaultPrevented).toBe(false);
      expect(query.value.rows.exec).toHaveLength(1);
    });

    it.each(["input", "textarea"])(
      "leaves query code pasted into an argument %s alone",
      (tag) => {
        const { query, first } = setup();
        const wrapper = mountRow(first);
        const field = document.createElement(tag);
        wrapper.find("[data-testid=query-row]").element.appendChild(field);
        const event = pasteEvent("search.elements");
        field.dispatchEvent(event);
        expect(event.defaultPrevented).toBe(false);
        expect(query.value.rows.exec).toHaveLength(1);
      },
    );

    it("ignores paste events without clipboard data", () => {
      const { query, first } = setup();
      const wrapper = mountRow(first);
      wrapper
        .find("[contenteditable]")
        .element.dispatchEvent(new Event("paste", { bubbles: true }));
      expect(query.value.rows.exec).toHaveLength(1);
    });

    it("does not paste while running", async () => {
      const { query, first } = setup();
      const wrapper = mountRow(first, { isRunning: true });
      expect(
        wrapper.find<HTMLButtonElement>(".paste-row-button").element.disabled,
      ).toBe(true);
      const event = pasteEvent(formatSteps(first.steps));
      wrapper.find("[data-testid=query-row]").element.dispatchEvent(event);
      expect(event.defaultPrevented).toBe(false);
      expect(query.value.rows.exec).toHaveLength(1);
    });
  });
});
