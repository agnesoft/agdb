import { mount } from "@vue/test-utils";
import AgdbCellMenu from "./AgdbCellMenu.vue";
import { describe, beforeEach, vi, it, expect } from "vitest";

describe("AgdbCellMenu", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("should open and close on click", () => {
        const wrapper = mount(AgdbCellMenu, {
            props: {
                row: {},
            },
        });
        const trigger = wrapper.find(".trigger");
        expect(wrapper.find(".content").isVisible()).toBe(false);
        trigger.trigger("click");
        expect(wrapper.find(".content").isVisible()).toBe(true);
        trigger.trigger("click");
        expect(wrapper.find(".content").isVisible()).toBe(false);
    });
});
