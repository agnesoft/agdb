import { AgdbApi } from "agdb_api";
import type { AxiosError } from "axios";

const client = await AgdbApi.client("http://localhost:3000").catch(
    (error: AxiosError) => {
        console.error(error.message);
        return undefined;
    },
);

const getLocalStorageToken = (): string | undefined => {
    const token = localStorage.getItem("token") ?? undefined;
    if (token) {
        client?.set_token(token);
    }
    return token;
};

const setLocalStorageToken = (token: string): void => {
    localStorage.setItem("token", token);
};

const getToken = (): string | undefined => {
    let token = client?.get_token();
    if (!token) {
        token = getLocalStorageToken();
    }
    return token;
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
