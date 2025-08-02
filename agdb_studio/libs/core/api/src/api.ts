import { AgdbApi } from "@agnesoft/agdb_api";
import type { AxiosError, AxiosResponse } from "axios";
import {
  ACCESS_TOKEN,
  BASE_CONNECTION_TIMEOUT,
  MAX_CONNECTION_ATTEMPTS,
} from "./constants";
import { computed, ref, type ComputedRef } from "vue";
import { addNotification } from "@agdb-studio/notification/src/composables/notificationStore.ts";
import type { AgdbApiClient } from "@agnesoft/agdb_api/client";
import { createLogger } from "@agdb-studio/utils/src/logger/logger";

const _client = ref<AgdbApi.AgdbApiClient | undefined>();

const logger = createLogger("ApiClient");

export const client = computed((): AgdbApi.AgdbApiClient | undefined => {
  return _client.value;
});

export const removeToken = (): void => {
  client.value?.reset_token();
  localStorage.removeItem(ACCESS_TOKEN);
  if (window.location.pathname !== "/studio/login") {
    window.location.reload();
  }
};

export const responseInterceptor = (response: AxiosResponse) => {
  logger.debug("Response Interceptor:", JSON.stringify(response));
  return response;
};

export const errorInterceptor = (error: AxiosError) => {
  logger.error(
    "Error Interceptor:",
    error.message,
    JSON.stringify(error.response),
    JSON.stringify(error.config),
  );
  if (error.response?.status === 401) {
    removeToken();
  }

  if (error.response) {
    addNotification({
      type: "error",
      title: `Error: ${error.response.statusText}`,
      message: `${error.response.data}`,
    });
  } else {
    addNotification({
      type: "error",
      title: "Error",
      message: `${error.message}`,
    });
  }
  return Promise.reject(error);
};

export const checkClient: (
  client: ComputedRef<AgdbApiClient | undefined>,
) => asserts client is ComputedRef<AgdbApiClient> = (client) => {
  if (!client.value) {
    throw new Error("Client is not initialized");
  }
};

let connectionAttempts = 0;

export const initClient = async (): Promise<void> => {
  _client.value = await AgdbApi.client(import.meta.env.VITE_API_URL).catch(
    (error: AxiosError) => {
      logger.error("Failed to initialize client:", error.message);
      if (connectionAttempts < MAX_CONNECTION_ATTEMPTS) {
        connectionAttempts++;
        const timeout = BASE_CONNECTION_TIMEOUT * connectionAttempts;
        let message = `Connection attempt ${connectionAttempts} failed. Retrying in ${timeout}ms.`;
        if (connectionAttempts === MAX_CONNECTION_ATTEMPTS) {
          message = `Connection attempt ${connectionAttempts} failed. Retrying in ${timeout}ms. This is the final attempt.`;
          addNotification({
            type: "error",
            title: "Connection error",
          });
        }
        logger.warn(message);
        setTimeout(() => {
          initClient();
        }, timeout);
      }
      return undefined;
    },
  );
  client.value?.interceptors.response.use(
    responseInterceptor,
    errorInterceptor,
  );
};
await initClient();
