import { describe, beforeEach, vi, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import AgdbMenu from "./AgdbMenu.vue";
import { dbActions } from "@/composables/db/dbConfig";
import { db_backup } from "@/tests/apiMock";

describe("AgdbMenu", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("should run action on click", () => {
        const wrapper = mount(AgdbMenu, {
            props: {
                actions: dbActions,
            },
        });

        expect(wrapper.find(".agdb-menu").exists()).toBe(true);
        expect(wrapper.find(".agdb-menu").text()).toContain("Convert");

        const backup = wrapper.find(".menu-item[data-key='backup']");
        backup.trigger("click");
        expect(db_backup).toHaveBeenCalled();
    });

    it("should render the sub menu on hover", async () => {
        const wrapper = mount(AgdbMenu, {
            props: {
                actions: dbActions,
            },
        });

        const convert = wrapper.find(".menu-item[data-key='convert']");
        await convert.trigger("mouseover");

        expect(wrapper.find(".sub-menu").exists()).toBe(true);
        expect(wrapper.find(".sub-menu").text()).toContain("Memory");

        await wrapper.trigger("mouseleave");

        expect(wrapper.find(".sub-menu").exists()).toBe(false);
    });
});
