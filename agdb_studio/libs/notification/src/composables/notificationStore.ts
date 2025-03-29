import { computed, ref, watch } from "vue";

export type AgdbNotificationType = "success" | "error" | "info" | "warning";

export type AgdbNotification = {
  type: AgdbNotificationType;
  title?: string;
  message?: string;
  id: string;
  timestamp: number;
  new?: boolean;
  read?: boolean;
};

const notifications = ref<AgdbNotification[]>([]);

const defaultTimeoutToHide = 3000;
let maxId = 0;

const viewerOpened = ref(false);
const toggleViewerOpened = (): void => {
  viewerOpened.value = !viewerOpened.value;
};
const closeViewer = (): void => {
  viewerOpened.value = false;
};

const markAllAsRead = (notif: AgdbNotification[]): void => {
  notif.forEach((notification) => {
    notification.read = true;
  });
};

watch(viewerOpened, (value) => {
  if (value) {
    markAllAsRead(notifications.value);
  }
});

const hideNotification = (index: string): void => {
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
  const id = `${maxId++}`;
  const timeout = notification.timeout ?? defaultTimeoutToHide;
  const notificationData = {
    ...notification,
    id,
    timestamp: Date.now(),
    new: true,
    read: false,
  };
  notifications.value.push(notificationData);
  setTimeout(() => {
    hideNotification(id);
  }, timeout);
  if (viewerOpened.value) {
    notificationData.read = true;
  }
};

const removeNotification = (index: string): void => {
  notifications.value = notifications.value.filter((item) => item.id !== index);
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
  closeViewer,
};
