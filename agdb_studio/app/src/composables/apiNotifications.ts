import { lastApiError, lastConnectionError } from "@agdb-studio/api/src/api";
import { addNotification } from "@agdb-studio/notification/src/composables/notificationStore";
import { watch } from "vue";

export const setupApiNotifications = (): void => {
  watch(lastApiError, (error) => {
    if (!error) {
      return;
    }

    addNotification({
      type: "error",
      title: `Error: ${error.status}`,
      message: error.description,
    });
  });

  watch(lastConnectionError, (errorMessage) => {
    if (!errorMessage) {
      return;
    }

    addNotification({
      type: "error",
      title: "Connection error",
      message: errorMessage,
    });
  });
};
