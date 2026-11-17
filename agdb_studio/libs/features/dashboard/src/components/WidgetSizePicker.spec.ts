import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import WidgetSizePicker from "./WidgetSizePicker.vue";

const mountPicker = () =>
  mount(WidgetSizePicker, {
    props: { size: { cols: 2, rows: 1 } },
    attachTo: document.body,
  });

const cell = (wrapper: ReturnType<typeof mountPicker>, size: string) =>
  wrapper.find(`[data-testid="widget-size-${size}"]`);

const covered = (wrapper: ReturnType<typeof mountPicker>) =>
  wrapper.findAll(".cell.covered").length;

describe("WidgetSizePicker", () => {
  it("shows the size and opens a grid of sizes", async () => {
    const wrapper = mountPicker();
    const toggle = wrapper.find('[data-testid="widget-size"]');
    expect(toggle.text()).toBe("2×1");
    expect(toggle.attributes("aria-expanded")).toBe("false");
    await toggle.trigger("click");
    expect(toggle.attributes("aria-expanded")).toBe("true");
    expect(wrapper.findAll(".cell")).toHaveLength(12);
    expect(cell(wrapper, "2x1").attributes("aria-pressed")).toBe("true");
    expect(cell(wrapper, "2x1").attributes("aria-label")).toBe(
      "2 wide, 1 high",
    );
    expect(covered(wrapper)).toBe(2);
    await toggle.trigger("click");
    expect(wrapper.find(".cells").exists()).toBe(false);
    wrapper.unmount();
  });

  it("previews the size under the pointer or focus and picks it", async () => {
    const wrapper = mountPicker();
    await wrapper.find('[data-testid="widget-size"]').trigger("click");
    await cell(wrapper, "3x2").trigger("mouseenter");
    expect(covered(wrapper)).toBe(6);
    expect(wrapper.find(".label").text()).toBe("3×2");
    await wrapper.find(".cells").trigger("mouseleave");
    expect(covered(wrapper)).toBe(2);
    await cell(wrapper, "4x3").trigger("focus");
    expect(covered(wrapper)).toBe(12);
    await cell(wrapper, "4x3").trigger("click");
    expect(wrapper.emitted("resize")).toEqual([[{ cols: 4, rows: 3 }]]);
    expect(wrapper.find(".cells").exists()).toBe(false);
    wrapper.unmount();
  });

  it("closes on Escape and on a click outside", async () => {
    const wrapper = mountPicker();
    const toggle = wrapper.find('[data-testid="widget-size"]');
    await toggle.trigger("click");
    await cell(wrapper, "1x1").trigger("keydown", { key: "Escape" });
    expect(wrapper.find(".cells").exists()).toBe(false);
    await toggle.trigger("click");
    // The outside click handler skips clicks in the same tick as the last one.
    await new Promise((resolve) => setTimeout(resolve));
    document.body.dispatchEvent(new Event("pointerdown", { bubbles: true }));
    document.body.click();
    await wrapper.vm.$nextTick();
    expect(wrapper.find(".cells").exists()).toBe(false);
    expect(wrapper.emitted("resize")).toBeUndefined();
    wrapper.unmount();
  });
});
