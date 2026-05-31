import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import AppLogo from "./AppLogo.vue";

describe("AppLogo", () => {
  it("renders the logo mark and label", async () => {
    const wrapper = mount(AppLogo);

    expect(wrapper.find("svg").exists()).toBe(true);
    expect(wrapper.text()).toContain("agdb");
  });
});
