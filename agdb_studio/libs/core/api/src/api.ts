import { AgdbApi } from "@agnesoft/agdb_api";
import type { AxiosError, AxiosResponse } from "axios";
import {
  ACCESS_TOKEN,
  BASE_CONNECTION_TIMEOUT,
  MAX_CONNECTION_ATTEMPTS,
} from "./constants";
import { computed, ref, type ComputedRef } from "vue";
import type { AgdbApiClient } from "@agnesoft/agdb_api/client";
import { createLogger } from "@agdb-studio/utils/src/logger/logger";

const _client = ref<AgdbApi.AgdbApiClient | undefined>();
const _apiUrl = ref(import.meta.env.VITE_API_URL);
const _lastApiError = ref<AxiosError | undefined>(undefined);
const _lastConnectionError = ref<string | undefined>(undefined);

const logger = createLogger("ApiClient");

export const client = computed((): AgdbApi.AgdbApiClient | undefined => {
  return _client.value;
});

export const apiUrl = computed((): string => {
  return _apiUrl.value;
});

export const lastApiError = computed((): AxiosError | undefined => {
  return _lastApiError.value;
});

export const lastConnectionError = computed((): string | undefined => {
  return _lastConnectionError.value;
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

  _lastApiError.value = error;

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

const attachInterceptors = (apiClient: AgdbApiClient): void => {
  apiClient.interceptors.response.use(responseInterceptor, errorInterceptor);
};

const connectToUrl = async (
  url: string,
): Promise<AgdbApiClient | undefined> => {
  return AgdbApi.client(url).catch((error: AxiosError) => {
    logger.error("Failed to initialize client:", error.message);
    if (connectionAttempts < MAX_CONNECTION_ATTEMPTS) {
      connectionAttempts++;
      const timeout = BASE_CONNECTION_TIMEOUT * connectionAttempts;
      let message = `Connection attempt ${connectionAttempts} failed. Retrying in ${timeout}ms.`;
      if (connectionAttempts === MAX_CONNECTION_ATTEMPTS) {
        message = `Connection attempt ${connectionAttempts} failed. Retrying in ${timeout}ms. This is the final attempt.`;
        _lastConnectionError.value = message;
      }
      logger.warn(message);
      setTimeout(() => {
        void initClient();
      }, timeout);
    }
    return undefined;
  });
};

export const initClient = async (): Promise<void> => {
  const nextClient = await connectToUrl(_apiUrl.value);
  _client.value = nextClient;
  if (nextClient) {
    _lastConnectionError.value = undefined;
    attachInterceptors(nextClient);
  }
};

export const reconnectClient = async (url: string): Promise<void> => {
  const token =
    client.value?.get_token() ??
    localStorage.getItem(ACCESS_TOKEN) ??
    undefined;
  const nextClient = await AgdbApi.client(url);

  attachInterceptors(nextClient);
  if (token) {
    nextClient.set_token(token);
  }

  _client.value = nextClient;
  _apiUrl.value = url;
  connectionAttempts = 0;
  _lastConnectionError.value = undefined;
};
await initClient();
