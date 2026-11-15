import { describe, beforeEach, afterEach, vi, it, expect } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import AgdbMenu from "./AgdbMenu.vue";

const submenuActions = [
  {
    key: "convert",
    label: "Convert",
    actions: [{ key: "memory", label: "Memory", action: vi.fn() }],
  },
];

const openSubmenuWithin = async (viewportWidth: number, right: number) => {
  vi.spyOn(document.documentElement, "clientWidth", "get").mockReturnValue(
    viewportWidth,
  );
  vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue({
    right,
  } as DOMRect);
  const wrapper = mount(AgdbMenu, { props: { actions: submenuActions } });
  await wrapper.find(".menu-item[data-key='convert']").trigger("mouseover");
  await flushPromises();
  return wrapper;
};

describe("AgdbMenu", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });
  afterEach(() => {
    vi.restoreAllMocks();
  });
  it("should run action on click", () => {
    const actionMock = vi.fn();
    const wrapper = mount(AgdbMenu, {
      props: {
        actions: [
          {
            key: "convert",
            label: "Convert",
          },
          {
            key: "backup",
            label: "Backup",
            action: actionMock,
          },
        ],
      },
    });

    expect(wrapper.find(".agdb-menu").exists()).toBe(true);
    expect(wrapper.find(".agdb-menu").text()).toContain("Convert");

    const backup = wrapper.find(".menu-item[data-key='backup']");
    backup.trigger("click");
    expect(actionMock).toHaveBeenCalled();
  });

  it("should render the sub menu on hover", async () => {
    const wrapper = mount(AgdbMenu, {
      props: {
        actions: [
          {
            key: "convert",
            label: "Convert",
            actions: [
              {
                key: "memory",
                label: "Memory",
                action: vi.fn(),
              },
              {
                key: "file",
                label: "File",
                action: vi.fn(),
              },
              {
                key: "mapped",
                label: "Mapped",
                action: vi.fn(),
              },
            ],
          },
        ],
      },
    });

    const convert = wrapper.find(".menu-item[data-key='convert']");
    await convert.trigger("mouseover");

    expect(wrapper.find(".sub-menu").exists()).toBe(true);
    expect(wrapper.find(".sub-menu").text()).toContain("Memory");

    await wrapper.trigger("mouseleave");

    expect(wrapper.find(".sub-menu").exists()).toBe(false);
  });

  it("should open the sub menu to the right when it fits", async () => {
    const wrapper = await openSubmenuWithin(1280, 1200);

    expect(wrapper.find(".sub-menu").classes()).not.toContain("sub-menu-left");
  });

  it("should open the sub menu to the left when it overflows the viewport", async () => {
    const wrapper = await openSubmenuWithin(1280, 1407);

    expect(wrapper.find(".sub-menu").classes()).toContain("sub-menu-left");
  });

  it("should not re-measure while the same sub menu stays open", async () => {
    const wrapper = await openSubmenuWithin(1280, 1407);
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue({
      right: 0,
    } as DOMRect);

    await wrapper.find(".menu-item[data-key='memory']").trigger("mouseover");
    await flushPromises();

    expect(wrapper.find(".sub-menu").classes()).toContain("sub-menu-left");
  });
});
