import { mount } from "@vue/test-utils";
import SpinnerIcon from "./SpinnerIcon.vue";
import LogoIcon from "./LogoIcon.vue";

describe("icons", () => {
  it("renders properly SpinnerIcon", () => {
    const wrapper = mount(SpinnerIcon);
    expect(wrapper.find("svg").exists()).toBe(true);
  });
  it("renders properly LogoIcon", () => {
    const wrapper = mount(LogoIcon);
    expect(wrapper.find("img").exists()).toBe(true);
  });
});
