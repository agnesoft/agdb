import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import ChartStat from "./ChartStat.vue";

describe("ChartStat", () => {
  it("shows the exact number and what it is", () => {
    const wrapper = mount(ChartStat, {
      props: { value: 1234.5, label: "Sum of price" },
    });
    expect(wrapper.find(".value").text()).toBe("1,234.5");
    expect(wrapper.find(".value").attributes("title")).toBe("1,234.5");
    expect(wrapper.find(".label").text()).toBe("Sum of price");
  });

  it("shows fractions and negative numbers in full", () => {
    const wrapper = mount(ChartStat, {
      props: { value: -0.25, label: "Min of change" },
    });
    expect(wrapper.find(".value").text()).toBe("-0.25");
  });
});
