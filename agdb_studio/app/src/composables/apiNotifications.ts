import { lastApiError, lastConnectionError } from "@agdb-studio/api/src/api";
import { addNotification } from "@agdb-studio/notification/src/composables/notificationStore";
import { watch } from "vue";

export const setupApiNotifications = (): void => {
  watch(lastApiError, (error) => {
    if (!error) {
      return;
    }

    if (error.response) {
      addNotification({
        type: "error",
        title: `Error: ${error.response.statusText}`,
        message: `${error.response.data}`,
      });
      return;
    }

    addNotification({
      type: "error",
      title: "Error",
      message: `${error.message}`,
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
