import { computed, ref } from "vue";

export type AgdbNotificationType = "success" | "error" | "info" | "warning";

export type AgdbNotification = {
    type: AgdbNotificationType;
    title?: string;
    message?: string;
    id: number;
    timestamp: number;
    new?: boolean;
};

const notifications = ref<AgdbNotification[]>([]);

const defaultTimeout = 5000;
let maxId = 0;

const hideNotification = (index: number): void => {
    const notification = notifications.value.find((item) => item.id === index);
    if (notification) {
        notification.new = false;
    }
};
const addNotification = (
    notification: Pick<AgdbNotification, "message" | "title" | "type"> & {
        timeout?: number;
    },
): void => {
    const id = maxId++;
    const timeout = notification.timeout ?? defaultTimeout;
    notifications.value.push({
        ...notification,
        id,
        timestamp: Date.now(),
        new: true,
    });
    setTimeout(() => {
        hideNotification(id);
    }, timeout);
};

const removeNotification = (index: number): void => {
    notifications.value.filter((_, i) => i !== index);
};

const clearNotifications = (): void => {
    notifications.value = [];
};

const newNotifications = computed(() => {
    return notifications.value.filter((item) => item.new);
});

export const useNotificationStore = () => {
    return {
        notifications,
        addNotification,
        removeNotification,
        clearNotifications,
        newNotifications,
    };
};
