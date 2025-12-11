import { mountSuspended } from "@nuxt/test-utils/runtime";
import DocsLayout from "./docs.vue";
import { describe, it, expect } from "vitest";

describe("DocsLayout", () => {
  it("mounts and renders aside", async () => {
    const wrapper = await mountSuspended(DocsLayout, {
      global: {
        provide: {
          navigation: [],
        },
        stubs: ["UContainer", "UPage", "UPageAside", "UContentNavigation"],
      },
    });
    expect(wrapper.findComponent({ name: "UPageAside" })).toBeTruthy();
  });
});
