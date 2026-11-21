import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import WidgetSizePicker from "./WidgetSizePicker.vue";

const mountPicker = () =>
  mount(WidgetSizePicker, { props: { size: { cols: 2, rows: 1 } } });

const cell = (wrapper: ReturnType<typeof mountPicker>, size: string) =>
  wrapper.find(`[data-testid="widget-size-${size}"]`);

const covered = (wrapper: ReturnType<typeof mountPicker>) =>
  wrapper.findAll(".cell.covered").length;

describe("WidgetSizePicker", () => {
  it("shows a grid of sizes with the current one", () => {
    const wrapper = mountPicker();
    expect(wrapper.findAll(".cell")).toHaveLength(12);
    expect(wrapper.find(".label").text()).toBe("2×1");
    expect(cell(wrapper, "2x1").attributes("aria-pressed")).toBe("true");
    expect(cell(wrapper, "2x1").attributes("aria-label")).toBe(
      "2 wide, 1 high",
    );
    expect(covered(wrapper)).toBe(2);
  });

  it("previews the size under the pointer or focus and picks it", async () => {
    const wrapper = mountPicker();
    await cell(wrapper, "3x2").trigger("mouseenter");
    expect(covered(wrapper)).toBe(6);
    expect(wrapper.find(".label").text()).toBe("3×2");
    await wrapper.find(".cells").trigger("mouseleave");
    expect(covered(wrapper)).toBe(2);
    await cell(wrapper, "4x3").trigger("focus");
    expect(covered(wrapper)).toBe(12);
    await wrapper.find(".cells").trigger("focusout");
    expect(covered(wrapper)).toBe(2);
    await cell(wrapper, "4x3").trigger("click");
    expect(wrapper.emitted("resize")).toEqual([[{ cols: 4, rows: 3 }]]);
  });
});
