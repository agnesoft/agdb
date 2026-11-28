import { describe, expect, it, vi } from "vitest";
import { ref } from "vue";

const box = vi.hoisted(() => ({ top: 0, bottom: 0, right: 0, width: 0 }));
const measured = vi.hoisted(() => ({ target: undefined as unknown }));
vi.mock("@vueuse/core", async () => {
  const { ref: vueRef, computed } = await import("vue");
  return {
    useElementBounding: (target: unknown) => {
      measured.target = target;
      return {
        top: computed(() => box.top),
        bottom: computed(() => box.bottom),
        right: computed(() => box.right),
      };
    },
    useElementSize: () => ({ width: computed(() => box.width) }),
    useWindowSize: () => ({ width: vueRef(1000), height: vueRef(800) }),
  };
});

import { useAnchoredPopover } from "./anchoredPopover";

const styleFor = (anchor: Partial<typeof box>) => {
  Object.assign(box, anchor);
  return useAnchoredPopover(ref(), ref()).value;
};

describe("useAnchoredPopover", () => {
  it("measures the anchor only while the popover is shown", () => {
    const anchor = ref(document.createElement("button"));
    const popover = ref<HTMLElement>();
    useAnchoredPopover(anchor, popover);
    const target = measured.target as { value: unknown };
    expect(target.value).toBeUndefined();
    popover.value = document.createElement("div");
    expect(target.value).toBe(anchor.value);
  });

  it("opens under the anchor, right-aligned, scrolling in the room below", () => {
    expect(styleFor({ top: 100, bottom: 130, right: 900, width: 400 })).toEqual(
      {
        right: "100px",
        top: "134px",
        maxWidth: "984px",
        maxHeight: "658px",
      },
    );
  });

  it("opens above the anchor when there is more room there", () => {
    expect(styleFor({ top: 700, bottom: 730, right: 900, width: 400 })).toEqual(
      {
        right: "100px",
        bottom: "104px",
        maxWidth: "984px",
        maxHeight: "688px",
      },
    );
  });

  it("stays inside the viewport", () => {
    // Near the left edge it moves right to fit.
    expect(styleFor({ top: 100, bottom: 130, right: 100 }).right).toBe("592px");
    expect(
      styleFor({ top: 100, bottom: 130, right: 995, width: 400 }).right,
    ).toBe("8px");
    // Before its width is known it sits right-aligned under the anchor.
    expect(
      styleFor({ top: 100, bottom: 130, right: 900, width: 0 }).right,
    ).toBe("100px");
    expect(
      styleFor({ top: 100, bottom: 130, right: 990, width: 2000 }).right,
    ).toBe("8px");
  });
});
