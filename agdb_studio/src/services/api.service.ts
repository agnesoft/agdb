import { AgdbApi } from "agdb_api";
import type { AxiosError, AxiosResponse } from "axios";
import {
    ACCESS_TOKEN,
    BASE_CONNECTION_TIMEOUT,
    MAX_CONNECTION_ATTEMPTS,
} from "@/constants";

let client: AgdbApi.AgdbApiClient | undefined;

export const getClient = (): AgdbApi.AgdbApiClient | undefined => {
    return client;
};

export const removeToken = (): void => {
    client?.reset_token();
    localStorage.removeItem(ACCESS_TOKEN);
    window.location.reload();
};

export const responseInterceptor = (response: AxiosResponse) => {
    return response;
};

export const errorInterceptor = (error: AxiosError) => {
    console.error(error.message);
    if (error.response?.status === 401) {
        removeToken();
    }
    return Promise.reject(error);
};

let connectionAttempts = 0;

export const initClient = async (): Promise<void> => {
    client = await AgdbApi.client(import.meta.env.VITE_API_URL).catch(
        (error: AxiosError) => {
            console.error(error.message);
            if (connectionAttempts < MAX_CONNECTION_ATTEMPTS) {
                connectionAttempts++;
                const timeout = BASE_CONNECTION_TIMEOUT * connectionAttempts;
                let message = `Connection attempt ${connectionAttempts} failed. Retrying in ${timeout}ms.`;
                if (connectionAttempts === MAX_CONNECTION_ATTEMPTS) {
                    message = `Connection attempt ${connectionAttempts} failed. Retrying in ${timeout}ms. This is the final attempt.`;
                }
                console.warn(message);
                setTimeout(() => {
                    initClient();
                }, timeout);
            }
            return undefined;
        },
    );
    console.log("Client initialized");
    client?.interceptors.response.use(responseInterceptor, errorInterceptor);
};
await initClient();
