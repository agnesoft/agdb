import { getClient } from "./api.service";
import { ACCESS_TOKEN } from "@/constants";

const setLocalStorageToken = (token: string): void => {
    localStorage.setItem(ACCESS_TOKEN, token);
};

const getToken = (): string | undefined => {
    const localStorageToken = localStorage.getItem(ACCESS_TOKEN);
    const clientToken = getClient()?.get_token();
    if (localStorageToken && clientToken !== localStorageToken) {
        getClient()?.set_token(localStorageToken);
    }
    return localStorageToken ?? undefined;
};

const removeToken = (): void => {
    getClient()?.reset_token();
    localStorage.removeItem(ACCESS_TOKEN);
    window.location.reload();
};

export const isLoggedIn = (): boolean => {
    return getToken() !== undefined;
};

export const login = async (
    username: string,
    password: string,
): Promise<string | undefined> => {
    return getClient()
        ?.login(username, password)
        .then((token) => {
            setLocalStorageToken(token);
            return token;
        });
};

export const logout = async (): Promise<void> => {
    if (!isLoggedIn()) {
        return;
    }
    await getClient()?.logout();
    removeToken();
};
