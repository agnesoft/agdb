import { describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { defineComponent, h, nextTick, ref } from "vue";
import { usePickerPopover } from "./pickerPopover";

const mountPopover = () => {
  const root = ref<HTMLElement>();
  const state: { popover?: ReturnType<typeof usePickerPopover> } = {};
  const wrapper = mount(
    defineComponent({
      setup() {
        state.popover = usePickerPopover(root);
        return () => h("div", { ref: root }, [h("span", { class: "inner" })]);
      },
    }),
    { attachTo: document.body },
  );
  return { wrapper, ...state.popover! };
};

describe("usePickerPopover", () => {
  it("closes on a pointerdown outside its root only", async () => {
    const { wrapper, open } = mountPopover();
    open.value = true;
    await nextTick();
    wrapper
      .get(".inner")
      .element.dispatchEvent(new MouseEvent("pointerdown", { bubbles: true }));
    expect(open.value).toBe(true);
    document.body.dispatchEvent(
      new MouseEvent("pointerdown", { bubbles: true }),
    );
    expect(open.value).toBe(false);
    wrapper.unmount();
  });

  it("takes Escape only while open", () => {
    const { wrapper, open, onEscape } = mountPopover();
    const closed = new KeyboardEvent("keydown", { key: "Escape" });
    const stopClosed = vi.spyOn(closed, "stopPropagation");
    onEscape(closed);
    expect(stopClosed).not.toHaveBeenCalled();
    open.value = true;
    const opened = new KeyboardEvent("keydown", { key: "Escape" });
    const stopOpened = vi.spyOn(opened, "stopPropagation");
    onEscape(opened);
    expect(stopOpened).toHaveBeenCalled();
    expect(open.value).toBe(false);
    wrapper.unmount();
  });

  it("stops listening once closed or unmounted", async () => {
    const remove = vi.spyOn(document, "removeEventListener");
    const { wrapper, open } = mountPopover();
    open.value = true;
    await nextTick();
    open.value = false;
    await nextTick();
    expect(remove).toHaveBeenCalledWith(
      "pointerdown",
      expect.any(Function),
      true,
    );
    remove.mockClear();
    wrapper.unmount();
    expect(remove).toHaveBeenCalledWith(
      "pointerdown",
      expect.any(Function),
      true,
    );
    remove.mockRestore();
  });
});
