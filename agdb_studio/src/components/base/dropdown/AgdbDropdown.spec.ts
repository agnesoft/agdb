import { mount } from "@vue/test-utils";
import AgdbDropdown from "./AgdbDropdown.vue";
import { describe, beforeEach, vi, it, expect } from "vitest";

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
        expect(wrapper.find(".content").exists()).toBe(false);
        trigger.trigger("click");
        await wrapper.vm.$nextTick();
        expect(wrapper.find(".content").isVisible()).toBe(true);
        trigger.trigger("click");
        await wrapper.vm.$nextTick();
        expect(wrapper.find(".content").exists()).toBe(false);
    });

    it("should close when clicking outside", async () => {
        const wrapper = mount(AgdbDropdown, {
            slots: {
                content: "<div>content</div>",
                trigger: "<div>trigger</div>",
            },
        });
        const trigger = wrapper.find(".trigger");
        expect(wrapper.find(".content").exists()).toBe(false);
        trigger.trigger("click");
        await wrapper.vm.$nextTick();
        expect(wrapper.find(".content").isVisible()).toBe(true);
        document.body.click();
        await wrapper.vm.$nextTick();
        document.body.click();
        await wrapper.vm.$nextTick();
        expect(wrapper.find(".content").exists()).toBe(false);
    });
});
