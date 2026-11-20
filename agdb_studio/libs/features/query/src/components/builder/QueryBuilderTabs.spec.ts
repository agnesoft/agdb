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

  it("starts on the exec tab and ignores arrow keys when given no tabs", async () => {
    const wrapper = mount(QueryBuilderTabs, { props: { tabs: [] } });
    expect(wrapper.findAll(".query-tab")).toHaveLength(0);
    const root = wrapper.find(".query-builder-tabs");
    await root.trigger("keydown", { key: "ArrowRight" });
    await root.trigger("keydown", { key: "ArrowLeft" });
    expect(wrapper.findAll(".query-tab")).toHaveLength(0);
  });

  it("shows only the given tabs and cycles through them", async () => {
    const wrapper = mount(QueryBuilderTabs, {
      props: { tabs: ["exec", "context"] },
    });
    const tabs = wrapper.findAll(".query-tab");
    expect(tabs.map((t) => t.text())).toEqual(["exec", "context"]);
    await wrapper.find(".query-builder-tabs").trigger("keydown", {
      key: "ArrowRight",
    });
    expect(wrapper.findAll(".query-tab")[1]!.classes()).toContain("active");
    await wrapper.find(".query-builder-tabs").trigger("keydown", {
      key: "ArrowRight",
    });
    expect(wrapper.findAll(".query-tab")[0]!.classes()).toContain("active");
  });
});
