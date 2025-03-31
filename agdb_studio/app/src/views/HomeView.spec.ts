import HomeView from "./HomeView.vue";
import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

describe("HomeView", () => {
  it("renders properly", () => {
    const wrapper = mount(HomeView);
    expect(wrapper.find(".main-wrapper")).toBeTruthy();
  });
});
