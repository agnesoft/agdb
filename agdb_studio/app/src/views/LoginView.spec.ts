import { describe, expect, it } from "vitest";
import LoginView from "./LoginView.vue";
import { mount } from "@vue/test-utils";

describe("LoginView", () => {
  it("renders properly", () => {
    const wrapper = mount(LoginView);
    expect(wrapper.text()).toContain("login");
  });
});
