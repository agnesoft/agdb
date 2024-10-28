import { AgdbApi } from "agdb_api";
import type { AxiosError, AxiosResponse } from "axios";

export const ACCESS_TOKEN = "studio_token";

let client: AgdbApi.AgdbApiClient | undefined;

export const getClient = (): AgdbApi.AgdbApiClient | undefined => {
    return client;
};

const getLocalStorageToken = (): string | undefined => {
    const token = localStorage.getItem(ACCESS_TOKEN) ?? undefined;
    if (token) {
        client?.set_token(token);
    }
    return token;
};

const setLocalStorageToken = (token: string): void => {
    localStorage.setItem(ACCESS_TOKEN, token);
};

const getToken = (): string | undefined => {
    return client?.get_token() ?? getLocalStorageToken();
};

const removeToken = (): void => {
    client?.reset_token();
    localStorage.removeItem(ACCESS_TOKEN);
    // window.location.reload();
};

export const isLoggedIn = (): boolean => {
    return getToken() !== undefined;
};

export const login = async (
    username: string,
    password: string,
): Promise<string | undefined> => {
    return client?.login(username, password).then((token) => {
        setLocalStorageToken(token);
        return token;
    });
};

export const logout = async (): Promise<void> => {
    if (!isLoggedIn()) {
        return;
    }
    await client?.logout();
    removeToken();
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
    client = await AgdbApi.client("http://localhost:3000").catch(
        (error: AxiosError) => {
            console.error(error.message);
            return undefined;
        },
    );
    client?.interceptors.response.use(responseInterceptor, errorInterceptor);
    return client;
};
await initClient();
