import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import RampPicker from "./RampPicker.vue";
import { GRAPH_THEME_FALLBACK } from "../../utils/graphStyle";

const mountPicker = () =>
  mount(RampPicker, {
    attachTo: document.body,
    props: {
      ramps: { ...GRAPH_THEME_FALLBACK.ramps, rose: ["#000000", "#ffffff"] },
      modelValue: "rose",
      label: "Scale",
    },
  });

describe("RampPicker", () => {
  it("shows the current scale and picks another", async () => {
    const wrapper = mountPicker();
    const trigger = wrapper.get("[data-testid=ramp-trigger]");
    expect(trigger.attributes("aria-label")).toBe("Scale: Rose");
    expect(trigger.attributes("style")).toContain("linear-gradient");

    await trigger.trigger("click");
    expect(trigger.attributes("aria-expanded")).toBe("true");
    const options = wrapper.findAll("[data-testid=ramp-option]");
    expect(options.map((option) => option.attributes("aria-pressed"))).toEqual([
      "false",
      "false",
      "true",
      "false",
    ]);
    await options[3]?.trigger("click");
    expect(wrapper.emitted("update:modelValue")).toEqual([["multi"]]);
    expect(wrapper.find("[data-testid=ramp-option]").exists()).toBe(false);
    wrapper.unmount();
  });

  it("closes on an outside pointerdown and on Esc", async () => {
    const wrapper = mountPicker();
    const trigger = wrapper.get("[data-testid=ramp-trigger]");
    await trigger.trigger("click");
    wrapper
      .get("[data-testid=ramp-option]")
      .element.dispatchEvent(new MouseEvent("pointerdown", { bubbles: true }));
    await wrapper.vm.$nextTick();
    expect(wrapper.find("[data-testid=ramp-option]").exists()).toBe(true);

    document.body.dispatchEvent(
      new MouseEvent("pointerdown", { bubbles: true }),
    );
    await wrapper.vm.$nextTick();
    expect(wrapper.find("[data-testid=ramp-option]").exists()).toBe(false);

    await trigger.trigger("click");
    await wrapper.trigger("keydown", { key: "Escape" });
    expect(wrapper.find("[data-testid=ramp-option]").exists()).toBe(false);
    wrapper.unmount();
  });
});
