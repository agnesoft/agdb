import { describe, expect, it, vi } from "vitest";
vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  return await importOriginal<typeof import("@agnesoft/agdb_api")>();
});
import { mount } from "@vue/test-utils";
import { nextTick } from "vue";
import ElementDetailsPanel from "./ElementDetailsPanel.vue";
import type { GraphLink, GraphNode } from "../../composables/types";

const node: GraphNode = {
  id: 1,
  values: [],
  rawValues: [
    { key: { String: "name" }, value: { String: "Alice" } },
    { key: { String: "age" }, value: { I64: 30 } },
  ],
  source: "query",
};

const link: GraphLink = {
  id: -2,
  values: [],
  rawValues: [],
  source: 1,
  target: 3,
  elementSource: "query",
};

const mountPanel = (props: Record<string, unknown> = {}) =>
  mount(ElementDetailsPanel, { props: { element: node, ...props } });

describe("ElementDetailsPanel", () => {
  it("shows node values with types in view mode", () => {
    const wrapper = mountPanel();
    expect(wrapper.text()).toContain("Node 1");
    const rows = wrapper.findAll("[data-testid=panel-value-row]");
    expect(rows).toHaveLength(2);
    expect(rows[1]?.text()).toContain("age");
    expect(rows[1]?.text()).toContain("30");
    expect(rows[1]?.text()).toContain("i64");
    expect(wrapper.find("[data-testid=panel-endpoints]").exists()).toBe(false);
  });

  it("shows edge endpoints and an empty state", () => {
    const wrapper = mountPanel({ element: link });
    expect(wrapper.text()).toContain("Edge -2");
    expect(wrapper.find("[data-testid=panel-endpoints]").text()).toContain(
      "From node 1 to node 3",
    );
    expect(wrapper.text()).toContain("No values.");
  });

  it("emits edit and close", async () => {
    const wrapper = mountPanel();
    await wrapper.find("[data-testid=panel-edit]").trigger("click");
    await wrapper.find("[data-testid=panel-close]").trigger("click");
    await wrapper.find("aside").trigger("keydown", { key: "Escape" });
    expect(wrapper.emitted("edit")).toHaveLength(1);
    expect(wrapper.emitted("close")).toHaveLength(2);
  });

  it("leaves edit mode on Esc, but keeps unsaved edits", async () => {
    const wrapper = mountPanel({ mode: "edit" });
    const esc = () =>
      wrapper.find("aside").trigger("keydown", { key: "Escape" });
    await esc();
    expect(wrapper.emitted("cancel")).toHaveLength(1);

    await wrapper.find("[data-testid=panel-value-input]").setValue("changed");
    expect(wrapper.emitted("dirty")).toEqual([[true]]);
    await esc();
    expect(wrapper.emitted("cancel")).toHaveLength(1);
    expect(wrapper.emitted("close")).toBeUndefined();
    expect(wrapper.find("[data-testid=panel-error]").text()).toContain(
      "Unsaved changes",
    );

    // Removing or adding rows counts as a change too.
    await wrapper.find("[data-testid=panel-value-input]").setValue("Alice");
    expect(wrapper.emitted("dirty")?.at(-1)).toEqual([false]);
    await wrapper.find("[data-testid=panel-remove-row]").trigger("click");
    expect(wrapper.emitted("dirty")?.at(-1)).toEqual([true]);
  });

  it("pre-fills the editor and saves typed values", async () => {
    const wrapper = mountPanel({ mode: "edit" });
    const rows = wrapper.findAll("[data-testid=panel-edit-row]");
    expect(rows).toHaveLength(2);
    const valueInputs = wrapper.findAll<HTMLInputElement>(
      "[data-testid=panel-value-input]",
    );
    expect(valueInputs[1]?.element.value).toBe("30");

    await valueInputs[1]?.setValue("31");
    await wrapper
      .findAll("[data-testid=panel-value-type]")[1]
      ?.setValue("unsigned");

    await wrapper.find("[data-testid=panel-add-row]").trigger("click");
    const keyInputs = wrapper.findAll("[data-testid=panel-key-input]");
    await keyInputs[2]?.setValue("city");
    await wrapper
      .findAll("[data-testid=panel-value-input]")[2]
      ?.setValue("Paris");

    await wrapper.find("form").trigger("submit");
    expect(wrapper.emitted("save")?.[0]?.[0]).toEqual([
      { key: { String: "name" }, value: { String: "Alice" } },
      { key: { String: "age" }, value: { U64: 31 } },
      { key: { String: "city" }, value: { String: "Paris" } },
    ]);
  });

  it("removes rows and emits cancel", async () => {
    const wrapper = mountPanel({ mode: "edit" });
    await wrapper.find("[data-testid=panel-remove-row]").trigger("click");
    expect(wrapper.findAll("[data-testid=panel-edit-row]")).toHaveLength(1);
    await wrapper.find("[data-testid=panel-cancel]").trigger("click");
    expect(wrapper.emitted("cancel")).toHaveLength(1);
  });

  it("validates missing keys, invalid values and duplicate keys", async () => {
    const wrapper = mountPanel({ mode: "edit" });
    const keyInputs = wrapper.findAll("[data-testid=panel-key-input]");
    const valueInputs = wrapper.findAll("[data-testid=panel-value-input]");
    const form = wrapper.find("form");

    await keyInputs[0]?.setValue(" ");
    await form.trigger("submit");
    expect(wrapper.find("[data-testid=panel-error]").text()).toBe(
      "Row 1: key is required",
    );

    await keyInputs[0]?.setValue("age");
    await form.trigger("submit");
    expect(wrapper.find("[data-testid=panel-error]").text()).toBe(
      "Row 2: duplicate key",
    );

    await keyInputs[0]?.setValue("name");
    await valueInputs[1]?.setValue("abc");
    await form.trigger("submit");
    expect(wrapper.find("[data-testid=panel-error]").text()).toContain(
      "Row 2 value: Expected an integer",
    );

    await wrapper.findAll("select")[0]?.setValue("signed");
    await form.trigger("submit");
    expect(wrapper.find("[data-testid=panel-error]").text()).toContain(
      "Row 1 key: Expected an integer",
    );
    expect(wrapper.emitted("save")).toBeUndefined();
  });

  it("resets the editor when the element changes", async () => {
    const wrapper = mountPanel({ mode: "edit" });
    await wrapper.find("[data-testid=panel-add-row]").trigger("click");
    expect(wrapper.findAll("[data-testid=panel-edit-row]")).toHaveLength(3);
    await wrapper.setProps({ element: link });
    await nextTick();
    expect(wrapper.findAll("[data-testid=panel-edit-row]")).toHaveLength(0);
  });

  it("keeps untouched values exactly, e.g. bytes", async () => {
    const element: GraphNode = {
      ...node,
      rawValues: [{ key: { String: "blob" }, value: { Bytes: [1, 2] } }],
    };
    const wrapper = mountPanel({ element, mode: "edit" });
    await wrapper.find("form").trigger("submit");
    expect(wrapper.emitted("save")?.[0]?.[0]).toEqual(element.rawValues);
  });

  it("saves elements with untouched integers above 2^53", async () => {
    const big = { U64: 1.7e18 }; // e.g. a nanosecond timestamp
    const element: GraphNode = {
      ...node,
      rawValues: [
        { key: { String: "ts" }, value: big },
        { key: { String: "name" }, value: { String: "Alice" } },
      ],
    };
    const wrapper = mountPanel({ element, mode: "edit" });
    await wrapper
      .findAll("[data-testid=panel-value-input]")[1]
      ?.setValue("Bob");
    await wrapper.find("form").trigger("submit");
    expect(wrapper.find("[data-testid=panel-error]").exists()).toBe(false);
    expect(wrapper.emitted("save")?.[0]?.[0]).toEqual([
      { key: { String: "ts" }, value: big },
      { key: { String: "name" }, value: { String: "Bob" } },
    ]);

    // Once edited, the large value is validated like any other.
    await wrapper
      .findAll("[data-testid=panel-value-input]")[0]
      ?.setValue("1700000000000000001");
    await wrapper.find("form").trigger("submit");
    expect(wrapper.find("[data-testid=panel-error]").text()).toContain(
      "Row 1 value: Expected a non-negative integer",
    );
  });

  it("still requires unique keys on untouched rows", async () => {
    const element: GraphNode = {
      ...node,
      rawValues: [{ key: { String: "name" }, value: { String: "Alice" } }],
    };
    const wrapper = mountPanel({ element, mode: "edit" });
    await wrapper.find("[data-testid=panel-add-row]").trigger("click");
    await wrapper.findAll("[data-testid=panel-key-input]")[1]?.setValue("name");
    await wrapper.find("form").trigger("submit");
    expect(wrapper.find("[data-testid=panel-error]").text()).toBe(
      "Row 2: duplicate key",
    );
  });

  it("emits the values from when editing started as the original", async () => {
    const wrapper = mountPanel({ mode: "edit" });
    const started = [...node.rawValues];
    await wrapper.setProps({
      element: {
        ...node,
        rawValues: [{ key: { String: "name" }, value: { String: "Changed" } }],
      },
    });
    await wrapper.find("form").trigger("submit");
    expect(wrapper.emitted("save")?.[0]?.[1]).toEqual(started);
  });

  it("shows nested keys read-only and keeps them when the value changes", async () => {
    const nestedKey = { VecDbValue: [{ String: "a" }, { I64: 1 }] };
    const element: GraphNode = {
      ...node,
      rawValues: [{ key: nestedKey, value: { String: "x" } }],
    };
    const wrapper = mountPanel({ element, mode: "edit" });
    const locked = wrapper.find<HTMLInputElement>(
      "[data-testid=panel-key-locked]",
    );
    expect(locked.element.readOnly).toBe(true);
    expect(locked.element.value).toBe("a, 1");
    expect(wrapper.find("[data-testid=panel-key-input]").exists()).toBe(false);
    await wrapper.find("[data-testid=panel-value-input]").setValue("y");
    await wrapper.find("form").trigger("submit");
    expect(wrapper.emitted("save")?.[0]?.[0]).toEqual([
      { key: nestedKey, value: { String: "y" } },
    ]);
  });

  it("does not require text for an empty nested key", async () => {
    const element: GraphNode = {
      ...node,
      rawValues: [{ key: { VecDbValue: [] }, value: { String: "x" } }],
    };
    const wrapper = mountPanel({ element, mode: "edit" });
    await wrapper.find("form").trigger("submit");
    expect(wrapper.emitted("save")).toHaveLength(1);
  });

  it("keeps edits in progress when the same element is rebuilt", async () => {
    const wrapper = mountPanel({ mode: "edit" });
    await wrapper
      .findAll("[data-testid=panel-value-input]")[0]
      ?.setValue("Bob");
    // Results changing elsewhere (e.g. expanding a node) rebuild the object.
    await wrapper.setProps({
      element: { ...node, rawValues: [...node.rawValues] },
    });
    await nextTick();
    const input = wrapper.find<HTMLInputElement>(
      "[data-testid=panel-value-input]",
    );
    expect(input.element.value).toBe("Bob");
  });

  it("resets the editor when the mode changes", async () => {
    const wrapper = mountPanel({ mode: "edit" });
    await wrapper.find("[data-testid=panel-add-row]").trigger("click");
    await wrapper.setProps({ mode: "view" });
    await wrapper.setProps({ mode: "edit" });
    expect(wrapper.findAll("[data-testid=panel-edit-row]")).toHaveLength(2);
  });

  it("keeps the original value when only the key is renamed", async () => {
    const element: GraphNode = {
      ...node,
      rawValues: [{ key: { String: "blob" }, value: { Bytes: [1, 2] } }],
    };
    const wrapper = mountPanel({ element, mode: "edit" });
    await wrapper.find("[data-testid=panel-key-input]").setValue("data");
    await wrapper.find("form").trigger("submit");
    expect(wrapper.emitted("save")?.[0]?.[0]).toEqual([
      { key: { String: "data" }, value: { Bytes: [1, 2] } },
    ]);
  });

  it("saves edited bytes as bytes", async () => {
    const element: GraphNode = {
      ...node,
      rawValues: [{ key: { String: "blob" }, value: { Bytes: [1, 2] } }],
    };
    const wrapper = mountPanel({ element, mode: "edit" });
    await wrapper.find("[data-testid=panel-value-input]").setValue("3, 255");
    await wrapper.find("form").trigger("submit");
    expect(wrapper.emitted("save")?.[0]?.[0]).toEqual([
      { key: { String: "blob" }, value: { Bytes: [3, 255] } },
    ]);
  });

  it("keeps a string that looks like a boolean a string", async () => {
    const element: GraphNode = {
      ...node,
      rawValues: [{ key: { String: "flag" }, value: { String: "true" } }],
    };
    const wrapper = mountPanel({ element, mode: "edit" });
    expect(
      wrapper.find<HTMLSelectElement>("[data-testid=panel-value-type]").element
        .value,
    ).toBe("string");
    await wrapper.find("[data-testid=panel-value-input]").setValue("false");
    await wrapper.find("form").trigger("submit");
    expect(wrapper.emitted("save")?.[0]?.[0]).toEqual([
      { key: { String: "flag" }, value: { String: "false" } },
    ]);
  });

  it("shows nested values read-only and keeps them when the key changes", async () => {
    const nested = { VecDbValue: [{ I64: 1 }, { String: "a" }] };
    const element: GraphNode = {
      ...node,
      rawValues: [{ key: { String: "mix" }, value: nested }],
    };
    const wrapper = mountPanel({ element, mode: "edit" });
    const locked = wrapper.find<HTMLInputElement>(
      "[data-testid=panel-value-locked]",
    );
    expect(locked.element.readOnly).toBe(true);
    expect(locked.element.value).toBe("1, a");
    expect(wrapper.find("[data-testid=panel-value-type]").exists()).toBe(false);
    await wrapper.find("[data-testid=panel-key-input]").setValue("mixed");
    await wrapper.find("form").trigger("submit");
    expect(wrapper.emitted("save")?.[0]?.[0]).toEqual([
      { key: { String: "mixed" }, value: nested },
    ]);
  });

  describe("stored values", () => {
    it("lists values only the query returned separately, read-only", async () => {
      const wrapper = mountPanel({
        element: {
          ...node,
          valuesState: "loaded",
          queryOnlyValues: [
            { key: { String: "edge_count" }, value: { U64: 2 } },
          ],
        },
      });
      expect(
        wrapper
          .findAll("[data-testid=panel-query-value-row]")
          .map((r) => r.text()),
      ).toEqual(["edge_count2u64"]);
      await wrapper.setProps({ mode: "edit" });
      // Only stored values are editable.
      expect(wrapper.findAll("[data-testid=panel-edit-row]")).toHaveLength(2);
    });

    it("waits for the stored values before editing", async () => {
      const wrapper = mountPanel({
        element: { ...node, rawValues: [], valuesState: "loading" },
        mode: "edit",
      });
      expect(wrapper.find("[data-testid=panel-values-loading]").exists()).toBe(
        true,
      );
      expect(wrapper.find("form").exists()).toBe(false);
      expect(
        wrapper.find<HTMLButtonElement>("[data-testid=panel-edit]").element
          .disabled,
      ).toBe(true);

      // Once loaded, the form is filled from the stored values.
      await wrapper.setProps({ element: { ...node, valuesState: "loaded" } });
      expect(wrapper.find("form").exists()).toBe(true);
      expect(wrapper.findAll("[data-testid=panel-edit-row]")).toHaveLength(2);
    });

    it("keeps the form while the stored values reload", async () => {
      const wrapper = mountPanel({
        element: { ...node, valuesState: "loaded" },
        mode: "edit",
      });
      await wrapper.find("[data-testid=panel-add-row]").trigger("click");
      await wrapper.setProps({ element: { ...node, valuesState: "loading" } });
      expect(wrapper.find("form").exists()).toBe(true);
      await wrapper.setProps({ element: { ...node, valuesState: "loaded" } });
      expect(wrapper.findAll("[data-testid=panel-edit-row]")).toHaveLength(3);
    });

    it("offers a retry when loading failed", async () => {
      const wrapper = mountPanel({
        element: { ...node, valuesState: "error" },
      });
      expect(wrapper.find("[data-testid=panel-values-error]").text()).toContain(
        "showing what the query returned",
      );
      await wrapper.find("[data-testid=panel-values-retry]").trigger("click");
      expect(wrapper.emitted("retry")).toHaveLength(1);
    });
  });
});
