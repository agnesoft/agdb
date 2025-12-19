import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import QueryBuilderTabs from "./QueryBuilderTabs.vue";

describe("QueryBuilderTabs", () => {
  it("renders the query builder tabs", () => {
    const wrapper = mount(QueryBuilderTabs);
    expect(wrapper.find(".query-builder-tabs").exists()).toBe(true);
    expect(wrapper.findAll(".button-tab").length).toBe(3);
    expect(wrapper.find(".button-tab.active").text()).toBe("exec");
  });
  it("switches tabs on click", async () => {
    const wrapper = mount(QueryBuilderTabs);
    const tabs = wrapper.findAll(".button-tab");
    await tabs[1]?.trigger("click");
    expect(wrapper.find(".button-tab.active").text()).toBe("exec_mut");
    await tabs[2]?.trigger("click");
    expect(wrapper.find(".button-tab.active").text()).toBe("context");
  });
});
