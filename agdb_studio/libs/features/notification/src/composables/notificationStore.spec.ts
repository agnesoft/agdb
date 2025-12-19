import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import {
  notifications,
  addNotification,
  removeNotification,
  clearNotifications,
  notificationsReversed,
  newNotifications,
  viewerOpened,
  toggleViewerOpened,
  hasUnreadNotifications,
  type AddNotificationProps,
  closeViewer,
} from "./notificationStore";
import { nextTick } from "vue";

const testNotifications: AddNotificationProps[] = [
  { type: "success", title: "Success", message: "This is a success message" },
  { type: "error", title: "Error", message: "This is an error message" },
  { type: "info", title: "Info", message: "This is an info message" },
  { type: "warning", title: "Warning", message: "This is a warning message" },
  { type: "success", message: "This is a success message without title" },
  { type: "error", message: "This is an error message without title" },
  { type: "info", message: "This is an info message without title" },
  {
    type: "warning",
    message:
      "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Ut iaculis felis et commodo lacinia. Aenean quis viverra velit, non pharetra metus. Sed scelerisque est eu purus condimentum malesuada. Mauris rutrum eros quis elit suscipit, suscipit tempus augue posuere. Aliquam pharetra porta ultricies. Nunc pellentesque volutpat urna, et ornare dolor posuere eu. Aliquam eget metus pharetra, tincidunt orci quis, efficitur justo. Curabitur posuere, velit sit amet efficitur molestie, eros diam finibus metus, eu tincidunt neque urna eu sem. Aliquam eros dui, rutrum eu eleifend at, volutpat hendrerit nulla. Sed eget fermentum eros. Integer turpis tellus, feugiat sit amet turpis eget, ornare dignissim justo.",
  },
  { type: "success", title: "Success without message" },
  { type: "error", title: "Error without message" },
  {
    type: "info",
    title:
      "Curabitur vulputate odio id diam placerat, nec ultricies tortor blandit. Suspendisse gravida dolor eget nunc pharetra tincidunt. Morbi sollicitudin est dui, a tempus massa maximus sit amet. Integer urna sapien, sollicitudin eget euismod scelerisque, efficitur in tellus. Nunc lectus ex, ornare non dui eget, sollicitudin ultrices urna. Maecenas vel laoreet ex. Pellentesque et urna egestas turpis maximus faucibus. Cras sapien metus, gravida ac neque in, hendrerit eleifend metus.",
  },
  { type: "warning", title: "Warning without message" },
];

describe("notificationStore", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    clearNotifications();
    closeViewer();
  });
  afterEach(() => {
    vi.useRealTimers();
  });
  it("adds notifications", () => {
    testNotifications.forEach((notification) => {
      addNotification(notification);
    });
    expect(notifications.value.length).toEqual(testNotifications.length);
    expect(newNotifications.value.length).toEqual(testNotifications.length);
    expect(notificationsReversed.value.length).toEqual(
      testNotifications.length,
    );

    for (let i = 0; i < testNotifications.length; i++) {
      expect(notifications.value[i]?.message).toEqual(
        testNotifications[i]?.message,
      );
      expect(notifications.value[i]?.title).toEqual(
        testNotifications[i]?.title,
      );
      expect(newNotifications.value[i]?.message).toEqual(
        testNotifications[testNotifications.length - 1 - i]?.message,
      );
      expect(newNotifications.value[i]?.title).toEqual(
        testNotifications[testNotifications.length - 1 - i]?.title,
      );
      expect(notificationsReversed.value[i]?.message).toEqual(
        testNotifications[testNotifications.length - 1 - i]?.message,
      );
      expect(notificationsReversed.value[i]?.title).toEqual(
        testNotifications[testNotifications.length - 1 - i]?.title,
      );
    }
  });

  it("removes notifications", () => {
    testNotifications.forEach((notification) => {
      addNotification(notification);
    });
    const id = notifications.value[1]?.id;
    expect(notifications.value.some((item) => item.id === id)).toEqual(true);
    removeNotification(notifications.value[1]?.id ?? "");
    expect(notifications.value.length).toEqual(testNotifications.length - 1);
    expect(notifications.value.some((item) => item.id === id)).toEqual(false);
    expect(newNotifications.value.length).toEqual(testNotifications.length - 1);
    expect(notificationsReversed.value.length).toEqual(
      testNotifications.length - 1,
    );
  });

  it("clears notifications", () => {
    testNotifications.forEach((notification) => {
      addNotification(notification);
    });
    clearNotifications();
    expect(notifications.value.length).toEqual(0);
    expect(newNotifications.value.length).toEqual(0);
    expect(notificationsReversed.value.length).toEqual(0);
  });

  it("toggles viewer", () => {
    expect(viewerOpened.value).toEqual(false);
    toggleViewerOpened();
    expect(viewerOpened.value).toEqual(true);
    toggleViewerOpened();
    expect(viewerOpened.value).toEqual(false);
  });

  it("checks for unread notifications and mark them read when viewer opens", async () => {
    expect(hasUnreadNotifications.value).toEqual(false);
    testNotifications.forEach((notification) => {
      addNotification(notification);
    });
    expect(hasUnreadNotifications.value).toEqual(true);

    toggleViewerOpened();
    await nextTick();
    expect(hasUnreadNotifications.value).toEqual(false);
  });

  it("marks all as read if viewer is opened", () => {
    toggleViewerOpened();
    testNotifications.forEach((notification) => {
      addNotification(notification);
    });
    expect(hasUnreadNotifications.value).toEqual(false);
  });

  it("marks notification as new and hides it after timeout", () => {
    addNotification({
      type: "success",
      title: "Success",
      message: "This is a success message",
      timeout: 100,
    });
    expect(newNotifications.value.length).toEqual(1);
    vi.advanceTimersByTime(101);
    expect(notifications.value.length).toEqual(1);
    expect(newNotifications.value.length).toEqual(0);
  });

  it("does not hide the notification if cleared before timeout", () => {
    addNotification({
      type: "success",
      title: "Success",
      message: "This is a success message",
      timeout: 100,
    });
    expect(newNotifications.value.length).toEqual(1);
    clearNotifications();
    vi.advanceTimersByTime(101);
    expect(notifications.value.length).toEqual(0);
    expect(newNotifications.value.length).toEqual(0);
  });
});
