import { mount } from "@vue/test-utils";
import { describe, beforeEach, vi, it, expect } from "vitest";
import NotificationViewer from "./NotificationViewer.vue";
import * as notificationStore from "@/composables/notification/notificationStore";

const toggleViewerOpenedMock = vi.spyOn(
    notificationStore,
    "toggleViewerOpened",
);
const closeViewerMock = vi.spyOn(notificationStore, "closeViewer");
const clearNotificationsMock = vi.spyOn(
    notificationStore,
    "clearNotifications",
);
const testNotifications: notificationStore.AddNotificationProps[] = [
    {
        type: "info",
        title: "Test",
        message: "This is a test notification",
    },
    {
        type: "info",
        title: "Test2",
        message: "This is a test notification 2",
    },
];

describe("NotificationViewer", () => {
    beforeEach(() => {
        notificationStore.clearNotifications();
        vi.clearAllMocks();
    });
    it("should close viewer when close button is clicked", async () => {
        notificationStore.addNotification(testNotifications[0]);
        notificationStore.viewerOpened.value = true;
        const wrapper = mount(NotificationViewer);
        expect(closeViewerMock).not.toHaveBeenCalled();
        await wrapper.find(".button-close").trigger("click");
        expect(closeViewerMock).toHaveBeenCalled();
    });

    it("should clear notifications when clear button is clicked", async () => {
        notificationStore.addNotification(testNotifications[0]);
        notificationStore.viewerOpened.value = true;
        const wrapper = mount(NotificationViewer);
        expect(clearNotificationsMock).not.toHaveBeenCalled();
        await wrapper.find(".button-clear").trigger("click");
        expect(clearNotificationsMock).toHaveBeenCalled();
    });

    it("should toggle viewer when notification-button is clicked", async () => {
        notificationStore.addNotification(testNotifications[0]);
        notificationStore.viewerOpened.value = false;
        const wrapper = mount(NotificationViewer);
        expect(toggleViewerOpenedMock).not.toHaveBeenCalled();
        await wrapper.find(".notification-button").trigger("click");
        expect(toggleViewerOpenedMock).toHaveBeenCalled();
    });

    it("should display new notifications when viewer is closed", async () => {
        notificationStore.addNotification(testNotifications[0]);
        notificationStore.viewerOpened.value = false;
        const wrapper = mount(NotificationViewer);
        notificationStore.addNotification(testNotifications[1]);
        await wrapper.vm.$nextTick();
        expect(wrapper.find(".notification-flash").text()).toContain(
            "This is a test notification 2",
        );
    });
});
