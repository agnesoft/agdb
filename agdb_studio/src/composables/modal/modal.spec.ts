import { describe, beforeEach, vi, it, expect } from "vitest";
import useModal from "./modal";

describe("Modal", () => {
    const { showModal, hideModal } = useModal();
    beforeEach(() => {
        hideModal();
    });
    it("shows a modal", () => {
        showModal({
            header: "Test Header",
            body: ["Test Body"],
        });
        expect(useModal().modalIsVisible.value).toBe(true);
    });
    it("hides a modal", () => {
        showModal({
            header: "Test Header",
            body: ["Test Body"],
        });
        hideModal();
        expect(useModal().modalIsVisible.value).toBe(false);
    });
    it("shows a modal with custom buttons", () => {
        showModal({
            header: "Test Header",
            body: ["Test Body"],
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
        showModal({
            header: "Test Header",
            body: ["Test Body"],
            onConfirm,
        });
        useModal().buttons.value[1].action();
        expect(onConfirm).toHaveBeenCalled();
        expect(useModal().modalIsVisible.value).toBe(false);
    });
    it("sets default if no header or body is provided", () => {
        showModal({});
        expect(useModal().content.header).toBe("");
        expect(useModal().content.body).toHaveLength(0);
    });
});
