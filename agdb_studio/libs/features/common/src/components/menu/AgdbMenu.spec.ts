import { describe, beforeEach, vi, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import AgdbMenu from "./AgdbMenu.vue";

describe("AgdbMenu", () => {
  beforeEach(() => {
    vi.clearAllMocks();
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
});
