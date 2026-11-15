import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import ColorSwatchPicker from "./ColorSwatchPicker.vue";

const palette = ["#111111", "#222222", "#333333"];

const mountPicker = (props: Record<string, unknown> = {}) =>
  mount(ColorSwatchPicker, {
    attachTo: document.body,
    props: { palette, label: "Colour", ...props },
  });

describe("ColorSwatchPicker", () => {
  it("opens the swatches from the trigger and picks one", async () => {
    const wrapper = mountPicker({ modelValue: 1 });
    const trigger = wrapper.get("[data-testid=swatch-trigger]");
    expect(trigger.classes()).not.toContain("empty");
    expect(wrapper.find("[data-testid=swatch]").exists()).toBe(false);

    await trigger.trigger("click");
    const swatches = wrapper.findAll("[data-testid=swatch]");
    expect(swatches).toHaveLength(3);
    expect(swatches[1]?.attributes("aria-pressed")).toBe("true");

    await swatches[2]?.trigger("click");
    expect(wrapper.emitted("update:modelValue")).toEqual([[2]]);
    expect(wrapper.find("[data-testid=swatch]").exists()).toBe(false);
    wrapper.unmount();
  });

  it("clears the colour, unless it is not clearable", async () => {
    const wrapper = mountPicker({ inline: true });
    expect(wrapper.find("[data-testid=swatch-trigger]").exists()).toBe(false);
    await wrapper.get("[data-testid=swatch-none]").trigger("click");
    expect(wrapper.emitted("update:modelValue")).toEqual([[undefined]]);

    await wrapper.setProps({ clearable: false });
    expect(wrapper.find("[data-testid=swatch-none]").exists()).toBe(false);
    wrapper.unmount();
  });

  it("closes on an outside pointerdown and on Esc", async () => {
    const wrapper = mountPicker();
    const trigger = wrapper.get("[data-testid=swatch-trigger]");
    expect(trigger.classes()).toContain("empty");

    await trigger.trigger("click");
    wrapper
      .get("[data-testid=swatch]")
      .element.dispatchEvent(new MouseEvent("pointerdown", { bubbles: true }));
    await wrapper.vm.$nextTick();
    expect(wrapper.find("[data-testid=swatch]").exists()).toBe(true);

    document.body.dispatchEvent(
      new MouseEvent("pointerdown", { bubbles: true }),
    );
    await wrapper.vm.$nextTick();
    expect(wrapper.find("[data-testid=swatch]").exists()).toBe(false);

    await trigger.trigger("click");
    await wrapper.trigger("keydown", { key: "Escape" });
    expect(wrapper.find("[data-testid=swatch]").exists()).toBe(false);
    wrapper.unmount();
  });
});
