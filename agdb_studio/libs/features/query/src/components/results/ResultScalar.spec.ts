import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import ResultScalar from "./ResultScalar.vue";

describe("ResultScalar", () => {
  it("renders the label and formatted value", () => {
    const wrapper = mount(ResultScalar, {
      props: { label: "Node count", value: 1234 },
    });
    expect(wrapper.text()).toContain("Node count");
    expect(wrapper.text()).toContain((1234).toLocaleString());
  });
});
