import { mount } from "@vue/test-utils";
import AgdbCellMenu from "./AgdbCellMenu.vue";
import { describe, beforeEach, vi, it, expect } from "vitest";
import { dbActions } from "@/composables/db/dbConfig";
import { INJECT_KEY_ROW } from "@/composables/table/constants";
const { fetchDatabases } = vi.hoisted(() => {
    return {
        fetchDatabases: vi.fn(),
    };
});

vi.mock("@/composables/db/DbStore", () => {
    return {
        useDbStore: () => {
            return {
                fetchDatabases,
            };
        },
    };
});
describe("AgdbCellMenu", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("should open and close on click", async () => {
        const wrapper = mount(AgdbCellMenu, {
            props: {
                actions: dbActions,
            },
            global: {
                provide: {
                    [INJECT_KEY_ROW]: {
                        value: {
                            role: "admin",
                            owner: "admin",
                            db: "test",
                            db_type: "memory",
                            size: 2656,
                            backup: 0,
                        },
                    },
                },
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
    it("should call action on click", async () => {
        const wrapper = mount(AgdbCellMenu, {
            props: {
                actions: dbActions,
            },
            global: {
                provide: {
                    [INJECT_KEY_ROW]: {
                        value: {
                            role: "admin",
                            owner: "admin",
                            db: "test",
                            db_type: "memory",
                            size: 2656,
                            backup: 0,
                        },
                    },
                },
            },
        });
        const trigger = wrapper.find(".trigger");
        expect(wrapper.find(".content").exists()).toBe(false);
        trigger.trigger("click");
        await wrapper.vm.$nextTick();
        expect(wrapper.find(".content").isVisible()).toBe(true);
        const action = wrapper.find(".menu-item[data-key=backup]");
        await action.trigger("click");
        await wrapper.vm.$nextTick();
        expect(wrapper.find(".content").exists()).toBe(false);
    });
    it("should not close the dropdown if item has no action", async () => {
        const wrapper = mount(AgdbCellMenu, {
            props: {
                actions: dbActions,
            },
            global: {
                provide: {
                    [INJECT_KEY_ROW]: {
                        value: {
                            role: "admin",
                            owner: "admin",
                            db: "test",
                            db_type: "memory",
                            size: 2656,
                            backup: 0,
                        },
                    },
                },
            },
        });
        const trigger = wrapper.find(".trigger");
        expect(wrapper.find(".content").exists()).toBe(false);
        trigger.trigger("click");
        await wrapper.vm.$nextTick();
        expect(wrapper.find(".content").isVisible()).toBe(true);
        const action = wrapper.find(".menu-item[data-key=convert]");
        await action.trigger("click");
        await wrapper.vm.$nextTick();
        expect(wrapper.find(".content").exists()).toBe(true);
    });
});
