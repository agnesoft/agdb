import { describe, beforeEach, vi, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import AgdbMenu from "./AgdbMenu.vue";
import type { Action } from "@/composables/content/types";
import type { TRow } from "../../composables/table/types";

const dbActions: Action<TRow>[] = [
  {
    key: "audit",
    label: "Audit",
    action: vi.fn(),
  },
  {
    key: "backup",
    label: "Backup",

    action: vi.fn(),
  },
];
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
            action: vi.fn(),
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
        actions: dbActions as unknown as Action<undefined>[],
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
