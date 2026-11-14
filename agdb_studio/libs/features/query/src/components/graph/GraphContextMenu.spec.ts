import { describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import GraphContextMenu from "./GraphContextMenu.vue";

const mountMenu = (props: Record<string, unknown> = {}) =>
  mount(GraphContextMenu, {
    attachTo: document.body,
    props: {
      x: 10,
      y: 20,
      title: "Node 1",
      actions: [{ key: "view", label: "View details", action: vi.fn() }],
      ...props,
    },
  });

describe("GraphContextMenu", () => {
  it("renders the actions at the given position", () => {
    const wrapper = mountMenu();
    const menu = document.querySelector<HTMLElement>(
      "[data-testid=graph-context-menu]",
    );
    expect(menu?.style.left).toBe("10px");
    expect(menu?.style.top).toBe("20px");
    expect(menu?.textContent).toContain("Node 1");
    expect(menu?.textContent).toContain("View details");
    wrapper.unmount();
  });

  it("clamps the position to the viewport", () => {
    const wrapper = mountMenu({ x: 100000, y: -5, title: "" });
    const menu = document.querySelector<HTMLElement>(
      "[data-testid=graph-context-menu]",
    );
    expect(menu?.style.left).toBe(`${window.innerWidth - 200}px`);
    expect(menu?.style.top).toBe("0px");
    expect(menu?.querySelector(".menu-title")).toBeNull();
    wrapper.unmount();
  });

  it("closes on Escape but not on other keys", () => {
    const wrapper = mountMenu();
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "a" }));
    expect(wrapper.emitted("close")).toBeUndefined();
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    expect(wrapper.emitted("close")).toHaveLength(1);
    wrapper.unmount();
  });

  it("closes on pointer down outside the menu only", () => {
    const wrapper = mountMenu();
    const menu = document.querySelector<HTMLElement>(
      "[data-testid=graph-context-menu]",
    );
    menu?.dispatchEvent(new Event("pointerdown", { bubbles: true }));
    expect(wrapper.emitted("close")).toBeUndefined();
    document.body.dispatchEvent(new Event("pointerdown", { bubbles: true }));
    expect(wrapper.emitted("close")).toHaveLength(1);
    wrapper.unmount();
  });

  it("removes listeners on unmount", () => {
    const wrapper = mountMenu();
    wrapper.unmount();
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    expect(wrapper.emitted("close")).toBeUndefined();
  });

  it("suppresses the native context menu on itself", () => {
    const wrapper = mountMenu();
    const event = new MouseEvent("contextmenu", { cancelable: true });
    document
      .querySelector("[data-testid=graph-context-menu]")
      ?.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
    wrapper.unmount();
  });
});
