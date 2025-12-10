import { mountSuspended } from "@nuxt/test-utils/runtime";
import AppHeader from "./AppHeader.vue";
import { describe, it, expect } from "vitest";

describe("AppHeader", () => {
  it("renders navigation links", async () => {
    const wrapper = await mountSuspended(AppHeader);
    const links = wrapper.findAll("a");
    expect(links.length).toBeGreaterThan(0);
  });
});
