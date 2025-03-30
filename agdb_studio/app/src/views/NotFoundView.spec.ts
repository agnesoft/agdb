import { vi, describe, it, beforeEach, expect } from "vitest";
import NotFoundView from "./NotFoundView.vue";
import { shallowMount } from "@vue/test-utils";

describe("NotFoundView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });
  it("renders the not found view", () => {
    const wrapper = shallowMount(NotFoundView);
    expect(wrapper.text()).toContain("404");
    expect(wrapper.text()).toContain("Page not found");
  });
});
