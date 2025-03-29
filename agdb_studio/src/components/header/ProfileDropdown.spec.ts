import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import ProfileDropdown from "./ProfileDropdown.vue";
import DropdownContent from "../base/dropdown/DropdownContent.vue";

vi.mock("@/composables/profile/account", () => {
  return {
    useAccount: () => ({
      username: "testUser",
      admin: { value: false },
    }),
  };
});

describe("ProfileDropdown", () => {
  it("renders", () => {
    const wrapper = mount(ProfileDropdown);
    expect(wrapper.text()).toContain("testUser");
  });

  it("should open and close on click", async () => {
    const wrapper = mount(ProfileDropdown);
    const trigger = wrapper.find(".trigger");
    const dropdown = wrapper.findComponent(DropdownContent);
    expect(dropdown.isVisible()).toBe(false);
    trigger.trigger("click");
    await wrapper.vm.$nextTick();
    expect(dropdown.isVisible()).toBe(true);
    trigger.trigger("click");
    await wrapper.vm.$nextTick();
    expect(dropdown.isVisible()).toBe(false);
  });
});
