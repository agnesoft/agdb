import { afterEach, describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { defineComponent, h } from "vue";
import { useGraphPalette } from "./graphPalette";

const listeners: (() => void)[] = [];
const removed = vi.fn();

const mountPalette = () => {
  let palette: ReturnType<typeof useGraphPalette> | undefined;
  const wrapper = mount(
    defineComponent({
      setup() {
        palette = useGraphPalette();
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
    const { wrapper, palette } = mountPalette();
    expect(palette()[0]).toBe("#010101");

    document.documentElement.style.setProperty("--graph-palette-1", "#020202");
    listeners.forEach((fn) => fn());
    expect(palette()[0]).toBe("#020202");

    wrapper.unmount();
    expect(removed).toHaveBeenCalled();
  });

  it("works without matchMedia", () => {
    vi.stubGlobal("matchMedia", undefined);
    const { wrapper, palette } = mountPalette();
    expect(palette().length).toBeGreaterThan(0);
    wrapper.unmount();
  });
});
