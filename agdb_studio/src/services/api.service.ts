import { AgdbApi } from "agdb_api";
import type { AxiosError, AxiosResponse } from "axios";
import { ACCESS_TOKEN } from "@/constants";

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

export const initClient = async () => {
    client = await AgdbApi.client(import.meta.env.VITE_API_URL).catch(
        (error: AxiosError) => {
            console.error(error.message);
            return undefined;
        },
    );
    client?.interceptors.response.use(responseInterceptor, errorInterceptor);
    return client;
};
await initClient();
