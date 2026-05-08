import { mountSuspended } from "@nuxt/test-utils/runtime";
import { describe, expect, it } from "vitest";
import AppLogo from "./AppLogo.vue";

describe("AppLogo", () => {
  it("renders the logo mark and label", async () => {
    const wrapper = await mountSuspended(AppLogo);

    expect(wrapper.find("svg").exists()).toBe(true);
    expect(wrapper.text()).toContain("agdb");
  });
});