import { mount } from "@vue/test-utils";
import AgdbCellMenu from "./AgdbCellMenu.vue";
import { describe, beforeEach, vi, it, expect } from "vitest";
import { dbActions } from "@/composables/db/dbConfig";
import { INJECT_KEY_ROW } from "@/composables/table/constants";
import useModal from "@/composables/modal/modal";
const { fetchDatabases } = vi.hoisted(() => {
    return {
        fetchDatabases: vi.fn(),
    };
});

vi.mock("@/composables/db/dbStore", () => {
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
    it("should call action on click when no confirmation required", async () => {
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
        const action = wrapper.find(".menu-item[data-key=audit]");
        await action.trigger("click");
        await wrapper.vm.$nextTick();
        expect(wrapper.find(".content").exists()).toBe(false);
    });
    it("should open the modal on click when confirmation is required", async () => {
        const deleteAction = vi.fn();
        const deleteConfirmation = [
            "Are you sure you want to delete this database?",
            "This will permanently delete all data.",
        ];
        const wrapper = mount(AgdbCellMenu, {
            props: {
                actions: [
                    {
                        key: "delete",
                        label: "Delete",
                        action: deleteAction,
                        confirmation: deleteConfirmation,
                    },
                ],
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
        const action = wrapper.find(".menu-item[data-key=delete]");
        await action.trigger("click");
        await wrapper.vm.$nextTick();
        expect(wrapper.find(".content").exists()).toBe(false);
        const { modalIsVisible, onConfirm } = useModal();
        expect(modalIsVisible.value).toBe(true);
        onConfirm.value?.();
        expect(deleteAction).toHaveBeenCalledOnce();
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
