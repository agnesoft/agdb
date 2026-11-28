import { afterEach, describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { defineComponent, h } from "vue";
import { useGraphPalette, useGraphRamps } from "./graphPalette";

const listeners: (() => void)[] = [];
const removed = vi.fn();

const mountPalette = <T>(use: () => { value: T }) => {
  let palette: { value: T } | undefined;
  const wrapper = mount(
    defineComponent({
      setup() {
        palette = use();
        return () => h("div");
      },
    }),
  );
  return { wrapper, palette: () => palette!.value };
};

describe("useGraphPalette", () => {
  afterEach(() => {
    document.documentElement.style.removeProperty("--graph-palette-1");
    vi.unstubAllGlobals();
    listeners.length = 0;
  });

  it("reads the palette and follows colour scheme changes", () => {
    vi.stubGlobal(
      "matchMedia",
      vi.fn(() => ({
        addEventListener: (_: string, fn: () => void) => listeners.push(fn),
        removeEventListener: removed,
      })),
    );
    document.documentElement.style.setProperty("--graph-palette-1", "#010101");
    const { wrapper, palette } = mountPalette(useGraphPalette);
    expect(palette()[0]).toBe("#010101");

    document.documentElement.style.setProperty("--graph-palette-1", "#020202");
    listeners.forEach((fn) => fn());
    expect(palette()[0]).toBe("#020202");

    wrapper.unmount();
    expect(removed).toHaveBeenCalled();
  });

  it("works without matchMedia", () => {
    vi.stubGlobal("matchMedia", undefined);
    const { wrapper, palette } = mountPalette(useGraphPalette);
    expect(palette().length).toBeGreaterThan(0);
    wrapper.unmount();
  });

  it("reads the gradient scales the same way", () => {
    vi.stubGlobal("matchMedia", undefined);
    document.documentElement.style.setProperty(
      "--graph-ramp-rose-1",
      "#030303",
    );
    const { wrapper, palette: ramps } = mountPalette(useGraphRamps);
    expect(ramps().rose[0]).toBe("#030303");
    wrapper.unmount();
    document.documentElement.style.removeProperty("--graph-ramp-rose-1");
  });
});
