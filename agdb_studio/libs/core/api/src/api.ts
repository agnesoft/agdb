import { AgdbApi, AgdbApiError, FetchClient } from "@agnesoft/agdb_api";
import {
  ACCESS_TOKEN,
  BASE_CONNECTION_TIMEOUT,
  MAX_CONNECTION_ATTEMPTS,
} from "./constants";
import { computed, shallowRef, type ComputedRef } from "vue";
import { createLogger } from "@agdb-studio/utils/src/logger/logger";

let _onUnauthorized: (() => void) | undefined;
let _isUnauthorized = false;

export const setOnUnauthorized = (callback: () => void): void => {
  _onUnauthorized = callback;
};

export const notifyUnauthorized = (): void => {
  if (_isUnauthorized) return;
  _isUnauthorized = true;
  _onUnauthorized?.();
};

export const resetUnauthorized = (): void => {
  _isUnauthorized = false;
};

const logger = createLogger("ApiClient");

/**
 * The transport of the studio: a 401 logs out and any other error is the
 * `lastApiError`, unless the client is quiet and its callers show their own.
 */
export class StudioClient extends FetchClient {
  constructor(public quiet: boolean = false) {
    super("agdb_studio");
  }

  override delete(uri: string, token: string | undefined): Promise<number> {
    return this.report(super.delete(uri, token));
  }

  override get<T>(
    uri: string,
    token: string | undefined,
  ): Promise<[number, T]> {
    return this.report(super.get<T>(uri, token));
  }

  override post<T, R>(
    uri: string,
    json: T | undefined,
    token: string | undefined,
  ): Promise<[number, R]> {
    return this.report(super.post<T, R>(uri, json, token));
  }

  override put<T>(
    uri: string,
    json: T | undefined,
    token: string | undefined,
  ): Promise<number> {
    return this.report(super.put<T>(uri, json, token));
  }

  private report<R>(response: Promise<R>): Promise<R> {
    return response.catch((error: unknown) => {
      onApiError(error, this.quiet);
      throw error;
    });
  }
}

export type StudioApi = AgdbApi<StudioClient>;

const _client = shallowRef<StudioApi | undefined>();
const _apiUrl = shallowRef<string>(import.meta.env.VITE_API_URL);
const _lastApiError = shallowRef<AgdbApiError | undefined>(undefined);
const _lastConnectionError = shallowRef<string | undefined>(undefined);

export const client = computed((): StudioApi | undefined => {
  return _client.value;
});

export const apiUrl = computed((): string => {
  return _apiUrl.value;
});

export const lastApiError = computed((): AgdbApiError | undefined => {
  return _lastApiError.value;
});

export const lastConnectionError = computed((): string | undefined => {
  return _lastConnectionError.value;
});

export const removeToken = (): void => {
  if (client.value) {
    client.value.token = undefined;
  }
  localStorage.removeItem(ACCESS_TOKEN);
  notifyUnauthorized();
};

const toApiError = (error: unknown): AgdbApiError =>
  error instanceof AgdbApiError ? error : new AgdbApiError(500, String(error));

export const onApiError = (error: unknown, quiet: boolean): void => {
  const apiError = toApiError(error);
  logger.error("Request failed:", apiError.to_string());

  if (apiError.status === 401) {
    removeToken();
    return;
  }

  if (!_isUnauthorized && !quiet) {
    _lastApiError.value = apiError;
  }
};

/**
 * A client of the same server and token as `api` whose errors are left to
 * its callers.
 */
export const quietClient = (api: StudioApi): StudioApi => {
  return new AgdbApi(
    new StudioClient(true),
    api.address(),
    api.base_url(),
    api.token,
  );
};

export const checkClient: (
  client: ComputedRef<StudioApi | undefined>,
) => asserts client is ComputedRef<StudioApi> = (client) => {
  if (!client.value) {
    throw new Error("Client is not initialized");
  }
};

let connectionAttempts = 0;

const connectToUrl = async (url: string): Promise<StudioApi | undefined> => {
  try {
    const api = AgdbApi.new(new StudioClient(true), url);
    await api.status();
    api.client.quiet = false;
    return api;
  } catch (error) {
    logger.error("Failed to connect:", toApiError(error).to_string());
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
  }
};

export const initClient = async (): Promise<void> => {
  const nextClient = await connectToUrl(_apiUrl.value);
  _client.value = nextClient;
  if (nextClient) {
    _lastConnectionError.value = undefined;
  }
};

export const reconnectClient = async (url: string): Promise<void> => {
  const token =
    client.value?.token ?? localStorage.getItem(ACCESS_TOKEN) ?? undefined;
  const nextClient = AgdbApi.new(new StudioClient(), url);

  if (token) {
    nextClient.token = token;
  }

  _client.value = nextClient;
  _apiUrl.value = url;
  connectionAttempts = 0;
  _lastConnectionError.value = undefined;
};
await initClient();
