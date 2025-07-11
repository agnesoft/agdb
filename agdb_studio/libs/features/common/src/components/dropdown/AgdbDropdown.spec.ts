import { mount } from "@vue/test-utils";
import AgdbDropdown from "./AgdbDropdown.vue";
import { describe, beforeEach, vi, it, expect } from "vitest";
import DropdownContent from "./DropdownContent.vue";

describe("AgdbDropdown", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });
  it("should open and close on click", async () => {
    const wrapper = mount(AgdbDropdown, {
      slots: {
        content: "<div>content</div>",
        trigger: "<div>trigger</div>",
      },
    });
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

  it("should close when clicking outside", async () => {
    const wrapper = mount(AgdbDropdown, {
      slots: {
        content: "<div>content</div>",
        trigger: "<div>trigger</div>",
      },
    });
    const trigger = wrapper.find(".trigger");
    const dropdown = wrapper.findComponent(DropdownContent);
    expect(dropdown.isVisible()).toBe(false);
    trigger.trigger("click");
    await wrapper.vm.$nextTick();
    expect(dropdown.isVisible()).toBe(true);
    document.body.click();
    await wrapper.vm.$nextTick();
    document.body.click();
    await wrapper.vm.$nextTick();
    expect(dropdown.isVisible()).toBe(false);
  });
});
