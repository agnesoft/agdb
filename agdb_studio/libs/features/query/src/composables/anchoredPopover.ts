import { computed, type Ref } from "vue";
import {
  useElementBounding,
  useElementSize,
  useWindowSize,
} from "@vueuse/core";

/** Space kept between the popover and the viewport edges. */
const MARGIN = 8;
/** Space between the anchor and the popover. */
const GAP = 4;
/** Below this much room under the anchor the popover opens above, if roomier. */
const MIN_BELOW = 240;

/**
 * Fixed-position style for a popover teleported out of a clipping container
 * (a small dashboard widget): right-aligned under `anchor`, flipped above when
 * there is more room there, kept inside the viewport and scrolling within it.
 * Anchored by its right edge, so it is in place before its width is known.
 */
export const useAnchoredPopover = (
  anchor: Ref<HTMLElement | undefined>,
  popover: Ref<HTMLElement | undefined>,
) => {
  // Measured only while the popover is shown, so a closed one costs nothing
  // on scroll, and measured afresh on opening, as the anchor may have moved.
  const box = useElementBounding(
    computed(() => (popover.value ? anchor.value : undefined)),
  );
  const { width: popoverWidth } = useElementSize(popover, undefined, {
    box: "border-box",
  });
  const viewport = useWindowSize();

  return computed(() => {
    const vw = viewport.width.value;
    const vh = viewport.height.value;
    const below = vh - box.bottom.value - GAP - MARGIN;
    const above = box.top.value - GAP - MARGIN;
    const up = below < MIN_BELOW && above > below;
    // Right-aligned with the anchor, but no further left than the margin.
    const right = Math.max(
      MARGIN,
      Math.min(vw - box.right.value, vw - MARGIN - popoverWidth.value),
    );
    return {
      right: `${right}px`,
      ...(up
        ? { bottom: `${vh - box.top.value + GAP}px` }
        : { top: `${box.bottom.value + GAP}px` }),
      maxWidth: `${vw - 2 * MARGIN}px`,
      maxHeight: `${Math.max(0, up ? above : below)}px`,
    };
  });
};
