import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import QueryGraph from "./QueryGraph.vue";

describe("QueryGraph", () => {
  it("renders the query graph", () => {
    const wrapper = mount(QueryGraph);
    expect(wrapper.find(".query-graph").exists()).toBe(true);
  });
});
