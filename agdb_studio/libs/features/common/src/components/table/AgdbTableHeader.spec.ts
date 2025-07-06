import { shallowMount } from "@vue/test-utils";
import AgdbTableHeader from "./AgdbTableHeader.vue";
import { describe, it, expect } from "vitest";
import { INJECT_KEY_TABLE_NAME } from "@/composables/table/constants";
import { TABLE_NAME } from "../../tests/tableMocks";

describe("TableHeader", () => {
  it("should render", () => {
    const wrapper = shallowMount(AgdbTableHeader, {
      props: {
        tableKey: "table",
      },
      global: {
        provide: {
          [INJECT_KEY_TABLE_NAME]: { value: TABLE_NAME },
        },
      },
    });
    expect(wrapper.exists()).toBe(true);
  });
  it("should handle if tableKey is undefined", () => {
    const wrapper = shallowMount(AgdbTableHeader, {
      global: {
        provide: {
          [INJECT_KEY_TABLE_NAME]: undefined,
        },
      },
    });
    expect(wrapper.exists()).toBe(true);
  });
});
