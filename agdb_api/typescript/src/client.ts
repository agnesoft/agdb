import { AxiosRequestConfig, OpenAPIClientAxios } from "openapi-client-axios";
import type { Client } from "./openapi";

type AgdbApi = {
    token: string | undefined;
    login: (username: string, password: string) => Promise<string>;  
    logout: () => Promise<void>;
    get_token: () => string | undefined;
    set_token: (token: string) => void;  
    reset_token: () => void;
};

async function login(username: string, password: string): Promise<string> {
    const token = await this.user_login(null, {
        username: username,
        password: password,
    });

    this.set_token(token.data);
    return token.data;
}

function get_token(): string | undefined {
    return this.token;
}

async function logout(): Promise<void> {
    await this.user_logout();
    this.reset_token();
}

function set_token(token: string): void {
    this.token = token;
    this.interceptors.request.use((config: AxiosRequestConfig) => {
        config.headers.Authorization = `Bearer ${token}`;
        return config;
    });
}

function reset_token(): void {
    this.token = undefined;
    this.interceptors.request.use((config: AxiosRequestConfig) => {
        return config;
    });
}

export type AgdbApiClient = Client & AgdbApi;

export async function client(address: string): Promise<AgdbApiClient> {
    const api: OpenAPIClientAxios = new OpenAPIClientAxios({
        withServer: { url: address },
        definition: `${address}/api/v1/openapi.json`,
    });
    const client = await api.init<AgdbApiClient>();
    client.token = undefined;
    client.login = login;
    client.logout = logout;
    client.set_token = set_token;
    client.get_token = get_token;
    client.reset_token = reset_token;
    return client;
}
