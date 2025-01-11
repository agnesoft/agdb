import { describe, beforeEach, vi, it, expect } from "vitest";
import useModal from "./modal";
import { convertArrayOfStringsToContent } from "@/composables/content/utils";
import { useContentInputs } from "../content/inputs";
import { KEY_MODAL } from "./constants";

describe("Modal", () => {
    const { openModal, closeModal } = useModal();
    beforeEach(() => {
        closeModal();
    });
    it("shows a modal", () => {
        openModal({
            header: "Test Header",
            content: convertArrayOfStringsToContent(["Test Body"]),
        });
        expect(useModal().modalIsVisible.value).toBe(true);
    });
    it("hides a modal", () => {
        openModal({
            header: "Test Header",
            content: convertArrayOfStringsToContent(["Test Body"]),
        });
        closeModal();
        expect(useModal().modalIsVisible.value).toBe(false);
    });
    it("shows a modal with custom buttons", () => {
        openModal({
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
        expect(useModal().buttons.value).toHaveLength(2);
        expect(useModal().buttons.value[0].text).toBe("Custom Button");
    });
    it("calls onConfirm when confirm button is clicked and hides the modal", () => {
        const onConfirm = vi.fn();
        openModal({
            header: "Test Header",
            content: convertArrayOfStringsToContent(["Test Body"]),
            onConfirm,
        });
        useModal().buttons.value[1].action();
        expect(onConfirm).toHaveBeenCalled();
        expect(useModal().modalIsVisible.value).toBe(false);
    });
    it("sets default if no header or content is provided", () => {
        openModal({});
        expect(useModal().modal.header).toBe("");
        expect(useModal().modal.content).toHaveLength(0);
    });
    it("adds inputs to the store", () => {
        const { getInputValue, setInputValue } = useContentInputs();
        setInputValue(KEY_MODAL, "test", "test");
        expect(getInputValue(KEY_MODAL, "test")).toBe(undefined);
        openModal({
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
        });
        setInputValue(KEY_MODAL, "test", "test");
        expect(getInputValue(KEY_MODAL, "test")).toBe("test");
    });
});
