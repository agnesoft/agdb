import { describe, expect, it, vi } from "vitest";
import LoginView from "./LoginView.vue";
import { mount } from "@vue/test-utils";

vi.mock("@agdb-studio/profile/src/components/LoginForm.vue", () => ({
  default: vi.fn(),
}));

describe("LoginView", () => {
  it("renders properly", () => {
    const wrapper = mount(LoginView);
    expect(wrapper.text()).toContain("login");
  });
});
