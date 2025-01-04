import { describe, beforeEach, it, expect } from "vitest";
import useModal from "@/composables/modal/modal";
import AgdbModal from "./AgdbModal.vue";
import { mount } from "@vue/test-utils";
import { convertArrayOfStringsToContent } from "@/composables/content/utils";

describe("AgdbModal", () => {
    const { showModal, hideModal } = useModal();
    beforeEach(() => {
        hideModal();
    });

    it("shows a modal when called", async () => {
        const wrapper = mount(AgdbModal, {
            attachTo: document.body,
        });
        expect(wrapper.isVisible()).toBe(false);
        showModal({
            header: "Test Header",
            content: convertArrayOfStringsToContent(["Test Body"]),
        });
        await wrapper.vm.$nextTick();
        expect(wrapper.isVisible()).toBe(true);
    });
    it("hides a modal when clicked on close button", async () => {
        const wrapper = mount(AgdbModal, {
            attachTo: document.body,
        });
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
        const wrapper = mount(AgdbModal, {
            attachTo: document.body,
        });
        showModal({
            header: "Test Header",
            content: convertArrayOfStringsToContent(["Test Body"]),
        });
        await wrapper.vm.$nextTick();
        await wrapper
            .find(".modal-header .button[data-testid=close-modal]")
            .trigger("click");
        await wrapper.vm.$nextTick();
        expect(wrapper.isVisible()).toBe(false);
    });
    it("shows a modal with custom buttons", async () => {
        const wrapper = mount(AgdbModal, {
            attachTo: document.body,
        });
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
        expect(wrapper.findAll(".modal-footer .button")).toHaveLength(2);
        expect(wrapper.find(".modal-footer .button").text()).toBe(
            "Custom Button",
        );
    });
    it("sets focus on the submit button", async () => {
        const wrapper = mount(AgdbModal, {
            attachTo: document.body,
        });
        showModal({
            header: "Test Header",
            content: convertArrayOfStringsToContent(["Test Body"]),
            onConfirm: () => {},
        });
        await wrapper.vm.$nextTick();
        await wrapper.vm.$nextTick();
        expect(
            wrapper.find(".button[type=submit]").element.matches(":focus"),
        ).toBe(true);
    });
    it("won't set focus on the submit button if content has input with autofocus", async () => {
        const wrapper = mount(AgdbModal, {
            attachTo: document.body,
        });
        showModal({
            header: "Test Header",
            content: [
                {
                    input: {
                        key: "test",
                        label: "New name",
                        type: "text",
                        autofocus: true,
                    },
                },
            ],
            onConfirm: () => {},
        });
        await wrapper.vm.$nextTick();
        await wrapper.vm.$nextTick();
        expect(
            wrapper.find(".button[type=submit]").element.matches(":focus"),
        ).toBe(false);
    });
});
