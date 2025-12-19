import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import QueryHinter from "./QueryHinter.vue";
import type { QueryType } from "../../composables/types";

describe("QueryHinter", () => {
  const hints: QueryType[] = ["select", "insert", "search"];

  it("renders all hints", () => {
    const wrapper = mount(QueryHinter, {
      props: { hints },
    });
    expect(wrapper.find(".query-hinter").exists()).toBe(true);
    expect(wrapper.findAll(".hinter-item").length).toBe(3);
    expect(wrapper.findAll(".hinter-item")[0]?.text()).toBe("select");
    expect(wrapper.findAll(".hinter-item")[1]?.text()).toBe("insert");
    expect(wrapper.findAll(".hinter-item")[2]?.text()).toBe("search");
  });

  it("highlights the active hint", () => {
    const wrapper = mount(QueryHinter, {
      props: { hints, activeIndex: 1 },
    });
    const items = wrapper.findAll(".hinter-item");
    expect(items[0]?.classes()).not.toContain("active");
    expect(items[1]?.classes()).toContain("active");
    expect(items[2]?.classes()).not.toContain("active");
  });

  it("emits selectHint when a hint is clicked", async () => {
    const wrapper = mount(QueryHinter, {
      props: { hints },
    });
    await wrapper.findAll(".hinter-item")[1]?.trigger("click");
    expect(wrapper.emitted("selectHint")).toBeTruthy();
    expect(wrapper.emitted("selectHint")?.[0]).toEqual(["insert"]);
  });

  it("renders empty when no hints provided", () => {
    const wrapper = mount(QueryHinter, {
      props: { hints: [] },
    });
    expect(wrapper.findAll(".hinter-item").length).toBe(0);
  });

  it("handles activeIndex outside bounds gracefully", () => {
    const wrapper = mount(QueryHinter, {
      props: { hints, activeIndex: 10 },
    });
    const activeItems = wrapper.findAll(".hinter-item.active");
    expect(activeItems.length).toBe(0);
  });
});
