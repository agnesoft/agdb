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
  it("switched tabs on arrow key press", async () => {
    const wrapper = mount(QueryBuilderTabs);
    const tabs = wrapper.findAll(".button-tab");

    // Initial active tab should be "exec"
    expect(wrapper.find(".button-tab.active").text()).toBe("exec");

    // Press right arrow key to go to "exec_mut"
    await tabs[0]?.trigger("keydown", { key: "ArrowRight" });
    expect(wrapper.find(".button-tab.active").text()).toBe("exec_mut");

    // Press right arrow key to go to "context"
    await tabs[1]?.trigger("keydown", { key: "ArrowRight" });
    expect(wrapper.find(".button-tab.active").text()).toBe("context");

    // Press left arrow key to go back to "exec_mut"
    await tabs[2]?.trigger("keydown", { key: "ArrowLeft" });
    expect(wrapper.find(".button-tab.active").text()).toBe("exec_mut");
  });
  it("does not switch tabs on non-arrow key press", async () => {
    const wrapper = mount(QueryBuilderTabs);
    const tabs = wrapper.findAll(".button-tab");

    // Initial active tab should be "exec"
    expect(wrapper.find(".button-tab.active").text()).toBe("exec");

    // Press a non-arrow key
    await tabs[0]?.trigger("keydown", { key: "Enter" });
    expect(wrapper.find(".button-tab.active").text()).toBe("exec");
  });
});
