import { vi, describe, it, beforeEach, expect } from "vitest";
import AdminView from "./AdminView.vue";
import { shallowMount, mount } from "@vue/test-utils";

describe("AdminView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });
  it("renders the admin db view", () => {
    const wrapper = shallowMount(AdminView);
    expect(wrapper.text()).toContain("Admin View");
  });
  it("renders a RouterLink to the database list (mount + real anchor stub)", () => {
    // Use mount and provide a simple RouterLink stub that renders an anchor
    const wrapper = mount(AdminView, {
      global: {
        stubs: {
          RouterLink: {
            props: ["to"],
            template: '<a :href="to"><slot /></a>',
          },
        },
      },
    });

    const link = wrapper.find("a");
    expect(link.exists()).toBe(true);
    expect(link.text()).toContain("Go to the database list");
    expect(link.attributes("href")).toBe("/admin/db");
  });
});
