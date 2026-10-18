import { describe, beforeEach, it, expect } from "vitest";
import useModal from "../../composables/modal/modal";
import AgdbModal from "./AgdbModal.vue";
import { mount, flushPromises } from "@vue/test-utils";
import { convertArrayOfStringsToContent } from "../../composables/content/utils";

describe("AgdbModal", () => {
  const { openModal, closeModal } = useModal();
  beforeEach(() => {
    closeModal();
  });

  it("shows a modal when called", async () => {
    const wrapper = mount(AgdbModal, {
      attachTo: document.body,
    });
    expect(wrapper.isVisible()).toBe(false);
    openModal({
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
    openModal({
      header: "Test Header",
      content: convertArrayOfStringsToContent(["Test Body"]),
    });
    await wrapper.vm.$nextTick();
    await wrapper.find(".modal-footer .button").trigger("click");
    await wrapper.vm.$nextTick();
    expect(wrapper.isVisible()).toBe(false);
  });
  it("hides a modal when clicked on close button in headers", async () => {
    const wrapper = mount(AgdbModal, {
      attachTo: document.body,
    });
    openModal({
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
    await wrapper.vm.$nextTick();
    expect(wrapper.findAll(".modal-footer .button")).toHaveLength(2);
    expect(wrapper.find(".modal-footer .button").text()).toBe("Custom Button");
  });
  it("sets focus on the submit button", async () => {
    const wrapper = mount(AgdbModal, {
      attachTo: document.body,
    });
    openModal({
      header: "Test Header",
      content: convertArrayOfStringsToContent(["Test Body"]),
      onConfirm: () => true,
    });
    await wrapper.vm.$nextTick();
    await wrapper.vm.$nextTick();
    expect(wrapper.find(".button[type=submit]").element.matches(":focus")).toBe(
      true,
    );
  });
  it("disables buttons and shows spinner while confirm is loading", async () => {
    let resolveConfirm: (value: boolean) => void = () => {};
    const wrapper = mount(AgdbModal, {
      attachTo: document.body,
    });
    openModal({
      header: "Test Header",
      content: convertArrayOfStringsToContent(["Test Body"]),
      onConfirm: () =>
        new Promise<boolean>((resolve) => {
          resolveConfirm = resolve;
        }),
    });
    await wrapper.vm.$nextTick();

    const confirmButton = wrapper.find(".modal-footer .button[type=submit]");
    await confirmButton.trigger("click");
    await wrapper.vm.$nextTick();

    const submitButtons = wrapper.findAll(".modal-footer .button[type=submit]");
    for (const btn of submitButtons) {
      expect(btn.attributes("disabled")).toBeDefined();
    }
    expect(
      wrapper
        .find(".modal-header .button[data-testid=close-modal]")
        .attributes("disabled"),
    ).toBeDefined();
    expect(wrapper.find(".spinner-icon").exists()).toBe(true);

    resolveConfirm(true);
    await flushPromises();

    for (const btn of wrapper.findAll(".modal-footer .button")) {
      expect(btn.attributes("disabled")).toBeUndefined();
    }
    expect(wrapper.find(".spinner-icon").exists()).toBe(false);
  });
  it("won't set focus on the submit button if content has input with autofocus", async () => {
    const wrapper = mount(AgdbModal, {
      attachTo: document.body,
    });
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
      onConfirm: () => true,
    });
    await wrapper.vm.$nextTick();
    await wrapper.vm.$nextTick();
    expect(wrapper.find(".button[type=submit]").element.matches(":focus")).toBe(
      false,
    );
  });
});
