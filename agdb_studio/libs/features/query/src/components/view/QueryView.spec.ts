import { describe, it, expect } from "vitest";
import QueryView from "./QueryView.vue";
import { mount } from "@vue/test-utils";

describe("QueryView", () => {
  it("renders the query view", () => {
    const wrapper = mount(QueryView, {
      global: {
        stubs: {
          QueryBuilder: {
            template:
              '<div class="mock-query-builder">Mock Query Builder</div>',
          },
          QueryGraph: {
            template: '<div class="mock-query-graph">Mock Query Graph</div>',
          },
        },
      },
    });
    expect(wrapper.find(".query-view").exists()).toBe(true);
    expect(wrapper.find(".mock-query-builder").exists()).toBe(true);
    expect(wrapper.find(".mock-query-graph").exists()).toBe(true);
  });
});
