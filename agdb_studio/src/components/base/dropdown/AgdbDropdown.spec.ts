import { mount } from "@vue/test-utils";
import AgdbDropdown from "./AgdbDropdown.vue";

describe("AgdbDropdown", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("should open and close on click", () => {
        const wrapper = mount(AgdbDropdown, {
            slots: {
                content: "<div>content</div>",
                trigger: "<div>trigger</div>",
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
