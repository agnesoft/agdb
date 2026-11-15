import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import InlineFieldEditor from "./InlineFieldEditor.vue";
import type { QueryStepFieldValue } from "../../composables/types";

const field = (): QueryStepFieldValue =>
  ({ selectedOption: "string", value: "a" }) as QueryStepFieldValue;

describe("InlineFieldEditor", () => {
  it("edits the type and the value of the field", async () => {
    const model = field();
    const wrapper = mount(InlineFieldEditor, {
      props: { modelValue: model, label: "Value", placeholder: "value" },
    });
    await wrapper.get("[data-testid=inline-type]").setValue("signed");
    await wrapper.get("[data-testid=inline-input]").setValue("12");
    expect(model).toMatchObject({ selectedOption: "signed", value: "12" });
    expect(
      wrapper.get("[data-testid=inline-input]").attributes("placeholder"),
    ).toBe("value");
  });

  it("commits on Enter and cancels on Esc", async () => {
    const wrapper = mount(InlineFieldEditor, {
      props: { modelValue: field(), label: "Value" },
    });
    await wrapper.trigger("keydown", { key: "Enter" });
    await wrapper.trigger("keydown", { key: "Escape" });
    expect(wrapper.emitted("commit")).toHaveLength(1);
    expect(wrapper.emitted("cancel")).toHaveLength(1);
  });

  it("focuses on mount when asked, and on demand", async () => {
    const wrapper = mount(InlineFieldEditor, {
      attachTo: document.body,
      props: { modelValue: field(), label: "Value", autofocus: true },
    });
    const input = wrapper.get<HTMLInputElement>(
      "[data-testid=inline-input]",
    ).element;
    expect(document.activeElement).toBe(input);
    input.blur();
    (wrapper.vm as unknown as { focus: () => void }).focus();
    expect(document.activeElement).toBe(input);
    wrapper.unmount();
  });

  it("disables its controls", () => {
    const wrapper = mount(InlineFieldEditor, {
      props: { modelValue: field(), label: "Value", disabled: true },
    });
    expect(
      wrapper.get("[data-testid=inline-input]").attributes("disabled"),
    ).toBeDefined();
  });
});
