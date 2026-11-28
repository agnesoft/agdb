import { onBeforeUnmount, ref, watch, type Ref } from "vue";

/**
 * Open state of a picker's popover inside `root`: a pointerdown outside it
 * closes it (captured, so a click that stops propagation still counts), and
 * `onEscape` closes it, keeping Escape from closing what is around it only
 * while it is open.
 */
export const usePickerPopover = (root: Ref<HTMLElement | undefined>) => {
  const open = ref(false);
  const onPointerDown = (event: PointerEvent) => {
    if (event.target instanceof Node && !root.value?.contains(event.target)) {
      open.value = false;
    }
  };
  watch(open, (isOpen) => {
    if (isOpen) document.addEventListener("pointerdown", onPointerDown, true);
    else document.removeEventListener("pointerdown", onPointerDown, true);
  });
  onBeforeUnmount(() =>
    document.removeEventListener("pointerdown", onPointerDown, true),
  );
  const onEscape = (event: KeyboardEvent) => {
    if (!open.value) return;
    event.stopPropagation();
    open.value = false;
  };
  return { open, onEscape };
};
