import { describe, beforeEach, it, expect } from "vitest";
import useModal from "@/composables/modal/modal";
import AgdbModal from "./AgdbModal.vue";
import { mount } from "@vue/test-utils";
import { convertArrayOfStringsToContent } from "@/utils/content";

describe("AgdbModal", () => {
    const { showModal, hideModal } = useModal();
    beforeEach(() => {
        hideModal();
    });
    const wrapper = mount(AgdbModal);

    it("shows a modal when called", async () => {
        expect(wrapper.isVisible()).toBe(false);
        showModal({
            header: "Test Header",
            content: convertArrayOfStringsToContent(["Test Body"]),
        });
        await wrapper.vm.$nextTick();
        expect(wrapper.isVisible()).toBe(true);
    });
    it("hides a modal when clicked on close button", async () => {
        showModal({
            header: "Test Header",
            content: convertArrayOfStringsToContent(["Test Body"]),
        });
        await wrapper.vm.$nextTick();
        await wrapper.find(".modal-footer .button").trigger("click");
        await wrapper.vm.$nextTick();
        expect(wrapper.isVisible()).toBe(false);
    });
    it("hides a modal when clicked on close button in heades", async () => {
        showModal({
            header: "Test Header",
            content: convertArrayOfStringsToContent(["Test Body"]),
        });
        await wrapper.vm.$nextTick();
        await wrapper.find(".modal-header .close-button").trigger("click");
        await wrapper.vm.$nextTick();
        expect(wrapper.isVisible()).toBe(false);
    });
    it("shows a modal with custom buttons", async () => {
        showModal({
            header: "Test Header",
            content: convertArrayOfStringsToContent(["Test Body"]),
            buttons: [
                {
                    className: "button",
                    text: "Custom Button",
                    action: () => {},
                },
            ],
        });
        await wrapper.vm.$nextTick();
        expect(wrapper.findAll(".button")).toHaveLength(2);
        expect(wrapper.find(".button").text()).toBe("Custom Button");
    });
});
