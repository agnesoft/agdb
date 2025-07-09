import { mount } from "@vue/test-utils";
import { describe, beforeEach, vi, it, expect } from "vitest";
import DropdownContent from "./DropdownContent.vue";

describe("DropdownContent", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });
  it("should render the content", () => {
    const wrapper = mount(DropdownContent, {
      props: {
        opened: true,
        buttonRef: undefined,
      },
      slots: {
        default: "<div>content</div>",
      },
    });
    expect(wrapper.html()).toContain("content");
  });
  it("should set the position of the dropdown", async () => {
    window.innerHeight = 100;
    window.innerWidth = 100;
    const buttonRef = document.createElement("div", {});
    buttonRef.getBoundingClientRect = () =>
      ({
        top: 50,
        left: 10,
        bottom: 20,
        right: 20,
      }) as DOMRect;
    vi.spyOn(buttonRef, "offsetTop", "get").mockReturnValue(80);
    const wrapper = mount(DropdownContent, {
      props: {
        opened: true,
        buttonRef,
      },
      slots: {
        default: "<div>content</div>",
      },
    });
    wrapper.element.getBoundingClientRect = () =>
      ({
        height: 60,
        width: 120,
      }) as DOMRect;
    await wrapper.vm.$nextTick();
    expect(wrapper.attributes("style")).toContain("left: -20px; top: 20px;");
  });
});
