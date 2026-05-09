import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import QueryArgumentDropdown from "./QueryArgumentDropdown.vue";

const VALUE_TYPE_SHORTCUTS: Record<string, string> = {
  string: "s",
  unsigned: "u",
  signed: "i",
  boolean: "b",
  float: "f",
  "string[]": "s[]",
  "unsigned[]": "u[]",
  "signed[]": "i[]",
  "boolean[]": "b[]",
  "float[]": "f[]",
};

const VALUE_TYPE_OPTIONS = Object.keys(VALUE_TYPE_SHORTCUTS);
const NON_VALUE_OPTIONS = ["asc", "desc"];

describe("QueryArgumentDropdown", () => {
  describe("closed state", () => {
    it("shows shortcut label for value-type field", () => {
      const wrapper = mount(QueryArgumentDropdown, {
        props: {
          options: VALUE_TYPE_OPTIONS,
          modelValue: "string",
          shortcuts: VALUE_TYPE_SHORTCUTS,
        },
      });
      expect(wrapper.find(".arg-select-label").text()).toBe("s");
    });

    it("shows empty string when modelValue is empty for value-type field", () => {
      const wrapper = mount(QueryArgumentDropdown, {
        props: {
          options: VALUE_TYPE_OPTIONS,
          modelValue: "",
          shortcuts: VALUE_TYPE_SHORTCUTS,
        },
      });
      expect(wrapper.find(".arg-select-label").text()).toBe("");
    });

    it("falls back to modelValue when shortcut is not in map", () => {
      const wrapper = mount(QueryArgumentDropdown, {
        props: {
          options: ["custom"],
          modelValue: "custom",
          shortcuts: {},
        },
      });
      expect(wrapper.find(".arg-select-label").text()).toBe("custom");
    });

    it("shows shortcut for array type option", () => {
      const wrapper = mount(QueryArgumentDropdown, {
        props: {
          options: VALUE_TYPE_OPTIONS,
          modelValue: "string[]",
          shortcuts: VALUE_TYPE_SHORTCUTS,
        },
      });
      expect(wrapper.find(".arg-select-label").text()).toBe("s[]");
    });

    it("shows full option name for non-value-type field", () => {
      const wrapper = mount(QueryArgumentDropdown, {
        props: {
          options: NON_VALUE_OPTIONS,
          modelValue: "asc",
        },
      });
      expect(wrapper.find(".arg-select-label").text()).toBe("asc");
    });

    it("dropdown list is not visible initially", () => {
      const wrapper = mount(QueryArgumentDropdown, {
        props: {
          options: NON_VALUE_OPTIONS,
          modelValue: "asc",
        },
      });
      expect(wrapper.find(".arg-options").exists()).toBe(false);
    });
  });

  describe("open state", () => {
    it("opens dropdown on trigger click", async () => {
      const wrapper = mount(QueryArgumentDropdown, {
        props: {
          options: NON_VALUE_OPTIONS,
          modelValue: "asc",
        },
      });
      await wrapper.find(".arg-select-trigger").trigger("click");
      expect(wrapper.find(".arg-options").exists()).toBe(true);
    });

    it("renders all options when open", async () => {
      const wrapper = mount(QueryArgumentDropdown, {
        props: {
          options: NON_VALUE_OPTIONS,
          modelValue: "asc",
        },
      });
      await wrapper.find(".arg-select-trigger").trigger("click");
      const options = wrapper.findAll(".arg-option");
      expect(options).toHaveLength(NON_VALUE_OPTIONS.length);
    });

    it("shows two columns (shortcut + full) for value-type options", async () => {
      const wrapper = mount(QueryArgumentDropdown, {
        props: {
          options: VALUE_TYPE_OPTIONS,
          modelValue: "string",
          shortcuts: VALUE_TYPE_SHORTCUTS,
        },
      });
      await wrapper.find(".arg-select-trigger").trigger("click");
      const firstOption = wrapper.findAll(".arg-option")[0];
      expect(firstOption?.find(".arg-option-shortcut").text()).toBe("s");
      expect(firstOption?.find(".arg-option-full").text()).toBe("string");
    });

    it("shows only full name column for non-value-type options", async () => {
      const wrapper = mount(QueryArgumentDropdown, {
        props: {
          options: NON_VALUE_OPTIONS,
          modelValue: "asc",
        },
      });
      await wrapper.find(".arg-select-trigger").trigger("click");
      const firstOption = wrapper.findAll(".arg-option")[0];
      expect(firstOption?.find(".arg-option-shortcut").exists()).toBe(false);
      expect(firstOption?.find(".arg-option-full").text()).toBe("asc");
    });

    it("marks the currently selected option", async () => {
      const wrapper = mount(QueryArgumentDropdown, {
        props: {
          options: NON_VALUE_OPTIONS,
          modelValue: "desc",
        },
      });
      await wrapper.find(".arg-select-trigger").trigger("click");
      const options = wrapper.findAll(".arg-option");
      expect(options[0]?.classes()).not.toContain("arg-option-selected");
      expect(options[1]?.classes()).toContain("arg-option-selected");
    });

    it("closes and emits value when option is clicked", async () => {
      const wrapper = mount(QueryArgumentDropdown, {
        props: {
          options: NON_VALUE_OPTIONS,
          modelValue: "asc",
        },
      });
      await wrapper.find(".arg-select-trigger").trigger("click");
      await wrapper.findAll(".arg-option")[1]?.trigger("click");
      expect(wrapper.emitted("update:modelValue")?.[0]).toEqual(["desc"]);
      expect(wrapper.find(".arg-options").exists()).toBe(false);
    });

    it("closes on Escape key", async () => {
      const wrapper = mount(QueryArgumentDropdown, {
        props: {
          options: NON_VALUE_OPTIONS,
          modelValue: "asc",
        },
      });
      await wrapper.find(".arg-select-trigger").trigger("click");
      expect(wrapper.find(".arg-options").exists()).toBe(true);
      await wrapper.trigger("keydown", { key: "Escape" });
      expect(wrapper.find(".arg-options").exists()).toBe(false);
    });

    it("falls back to opt when shortcut is missing for an option in open list", async () => {
      const wrapper = mount(QueryArgumentDropdown, {
        props: {
          options: ["custom"],
          modelValue: "custom",
          shortcuts: {},
        },
      });
      await wrapper.find(".arg-select-trigger").trigger("click");
      const opt = wrapper.find(".arg-option-full");
      expect(wrapper.find(".arg-option-shortcut").exists()).toBe(false);
      expect(opt.text()).toBe("custom");
    });

    it("toggles closed when trigger is clicked again", async () => {
      const wrapper = mount(QueryArgumentDropdown, {
        props: {
          options: NON_VALUE_OPTIONS,
          modelValue: "asc",
        },
      });
      await wrapper.find(".arg-select-trigger").trigger("click");
      await wrapper.find(".arg-select-trigger").trigger("click");
      expect(wrapper.find(".arg-options").exists()).toBe(false);
    });

    it("shows option name for options without a shortcut when shortcuts exist for others", async () => {
      const wrapper = mount(QueryArgumentDropdown, {
        props: {
          options: ["string", "custom"],
          modelValue: "string",
          shortcuts: { string: "s" },
        },
      });
      await wrapper.find(".arg-select-trigger").trigger("click");
      const options = wrapper.findAll(".arg-option");
      // "custom" has no shortcut, falls back to option name
      expect(options[1]?.find(".arg-option-shortcut").text()).toBe("custom");
    });
  });
});
