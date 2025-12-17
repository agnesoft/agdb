import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import QueryBuilder from "./QueryBuilder.vue";

describe("QueryBuilder", () => {
  it("renders the query builder", () => {
    const wrapper = mount(QueryBuilder);
    expect(wrapper.find(".query-builder").exists()).toBe(true);
    expect(wrapper.find(".query-input").exists()).toBe(true);
    expect(wrapper.find("button.button-primary").exists()).toBe(true);
  });
});
