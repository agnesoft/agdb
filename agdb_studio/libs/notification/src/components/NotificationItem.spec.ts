import { mount } from "@vue/test-utils";
import { describe, beforeEach, vi, it, expect } from "vitest";
import NotificationItem from "./NotificationItem.vue";

const { removeNotification } = vi.hoisted(() => {
  return {
    removeNotification: vi.fn(),
  };
});

vi.mock("@/composables/notification/notificationStore", () => {
  return {
    removeNotification,
  };
});

describe("NotificationItem", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });
  it("should remove notification when close button is clicked", async () => {
    const wrapper = mount(NotificationItem, {
      props: {
        notification: {
          id: "testId",
          type: "info",
          title: "Test",
          message: "This is a test notification",
          timestamp: Date.now(),
        },
      },
    });
    expect(removeNotification).not.toHaveBeenCalled();
    await wrapper.find(".button-close").trigger("click");
    expect(removeNotification).toHaveBeenLastCalledWith("testId");
  });

  it("should display separator when title and message are provided", () => {
    const wrapper = mount(NotificationItem, {
      props: {
        notification: {
          id: "testId",
          type: "info",
          title: "Test",
          message: "This is a test notification",
          timestamp: Date.now(),
        },
      },
    });
    expect(wrapper.find(".separator").exists()).toBe(true);
  });

  it("should not display separator when only title is provided", () => {
    const wrapper = mount(NotificationItem, {
      props: {
        notification: {
          id: "testId",
          type: "info",
          title: "Test",
          timestamp: Date.now(),
        },
      },
    });
    expect(wrapper.find(".separator").exists()).toBe(false);
  });

  it("should not display separator when only message is provided", () => {
    const wrapper = mount(NotificationItem, {
      props: {
        notification: {
          id: "testId",
          type: "info",
          message: "This is a test notification",
          timestamp: Date.now(),
        },
      },
    });
    expect(wrapper.find(".separator").exists()).toBe(false);
  });
});
