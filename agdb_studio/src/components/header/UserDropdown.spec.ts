import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import UserDropdown from "./UserDropdown.vue";
import DropdownContent from "../base/dropdown/DropdownContent.vue";

vi.mock("@/composables/user/account", () => {
    return {
        useAccount: () => ({
            username: "testUser",
        }),
    };
});

describe("UserDropdown", () => {
    it("renders", () => {
        const wrapper = mount(UserDropdown);
        expect(wrapper.text()).toContain("testUser");
    });

    it("should open and close on click", async () => {
        const wrapper = mount(UserDropdown);
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
