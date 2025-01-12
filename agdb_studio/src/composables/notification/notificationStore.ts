import { computed, ref, watch } from "vue";

export type AgdbNotificationType = "success" | "error" | "info" | "warning";

export type AgdbNotification = {
    type: AgdbNotificationType;
    title?: string;
    message?: string;
    id: number;
    timestamp: number;
    new?: boolean;
    read?: boolean;
};

const notifications = ref<AgdbNotification[]>([]);

const defaultTimeoutToHide = 3000;
const timeoutToRead = 1000;
let maxId = 0;

const viewerOpened = ref(false);
const toggleViewerOpened = (): void => {
    viewerOpened.value = !viewerOpened.value;
};

const markAsRead = (index: number): void => {
    setTimeout(() => {
        const notification = notifications.value.find(
            (item) => item.id === index,
        );
        if (notification) {
            notification.read = true;
        }
    }, timeoutToRead);
};

const markAllAsRead = (notif: AgdbNotification[]): void => {
    setTimeout(() => {
        notif.forEach((notification) => {
            notification.read = true;
        });
    }, timeoutToRead);
};

watch(viewerOpened, (value) => {
    if (value) {
        markAllAsRead(notifications.value);
    }
});

const hideNotification = (index: number): void => {
    const notification = notifications.value.find((item) => item.id === index);
    if (notification) {
        notification.new = false;
    }
};

export type AddNotificationProps = Pick<
    AgdbNotification,
    "message" | "title" | "type"
> & {
    timeout?: number;
};

const addNotification = (notification: AddNotificationProps): void => {
    const id = maxId++;
    const timeout = notification.timeout ?? defaultTimeoutToHide;
    notifications.value.push({
        ...notification,
        id,
        timestamp: Date.now(),
        new: true,
        read: false,
    });
    setTimeout(() => {
        hideNotification(id);
    }, timeout);
    if (viewerOpened.value) {
        markAsRead(id);
    }
};

const removeNotification = (index: number): void => {
    console.log("removeNotification", index);
    notifications.value = notifications.value.filter(
        (item) => item.id !== index,
    );
    console.log(notifications.value);
};

const clearNotifications = (): void => {
    notifications.value = [];
};

const notificationsReversed = computed(() => {
    return [...notifications.value].reverse();
});

const newNotifications = computed(() => {
    return [...notifications.value.filter((item) => item.new)].reverse();
});

const hasUnreadNotifications = computed(() => {
    return notifications.value.some((item) => !item.read);
});

// const testNotifications: AddNotificationProps[] = [
//     { type: "success", title: "Success", message: "This is a success message" },
//     { type: "error", title: "Error", message: "This is an error message" },
//     { type: "info", title: "Info", message: "This is an info message" },
//     { type: "warning", title: "Warning", message: "This is a warning message" },
//     { type: "success", message: "This is a success message without title" },
//     { type: "error", message: "This is an error message without title" },
//     { type: "info", message: "This is an info message without title" },
//     {
//         type: "warning",
//         message:
//             "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Ut iaculis felis et commodo lacinia. Aenean quis viverra velit, non pharetra metus. Sed scelerisque est eu purus condimentum malesuada. Mauris rutrum eros quis elit suscipit, suscipit tempus augue posuere. Aliquam pharetra porta ultricies. Nunc pellentesque volutpat urna, et ornare dolor posuere eu. Aliquam eget metus pharetra, tincidunt orci quis, efficitur justo. Curabitur posuere, velit sit amet efficitur molestie, eros diam finibus metus, eu tincidunt neque urna eu sem. Aliquam eros dui, rutrum eu eleifend at, volutpat hendrerit nulla. Sed eget fermentum eros. Integer turpis tellus, feugiat sit amet turpis eget, ornare dignissim justo.",
//     },
//     { type: "success", title: "Success without message" },
//     { type: "error", title: "Error without message" },
//     {
//         type: "info",
//         title: "Curabitur vulputate odio id diam placerat, nec ultricies tortor blandit. Suspendisse gravida dolor eget nunc pharetra tincidunt. Morbi sollicitudin est dui, a tempus massa maximus sit amet. Integer urna sapien, sollicitudin eget euismod scelerisque, efficitur in tellus. Nunc lectus ex, ornare non dui eget, sollicitudin ultrices urna. Maecenas vel laoreet ex. Pellentesque et urna egestas turpis maximus faucibus. Cras sapien metus, gravida ac neque in, hendrerit eleifend metus.",
//     },
//     { type: "warning", title: "Warning without message" },
// ];

// const testAddNotification = () => {
//     testNotifications.forEach((notification) => {
//         addNotification(notification);
//     });
// };
// testAddNotification();

export {
    notifications,
    addNotification,
    removeNotification,
    clearNotifications,
    notificationsReversed,
    newNotifications,
    viewerOpened,
    toggleViewerOpened,
    hasUnreadNotifications,
};
