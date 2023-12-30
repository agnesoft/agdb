import type { Client } from "../../../agdb_api/typescript/src/schema";
import OpenAPIClientAxios from "openapi-client-axios";
import { AxiosResponse, InternalAxiosRequestConfig } from "axios";

export class Api {
    private static api: OpenAPIClientAxios | undefined = undefined;
    private static c: Client | undefined = undefined;
    private static token: string = "";

    static async client(): Promise<Client> {
        if (Api.c === undefined) {
            Api.api = new OpenAPIClientAxios({
                definition: `${import.meta.env.VITE_API_URL}/api/v1/openapi.json`,
            });

            Api.c = await Api.api.init<Client>();
            Api.c.interceptors.request.use((config: InternalAxiosRequestConfig) => {
                if (Api.token !== "") {
                    config.headers.Authorization = `Bearer ${Api.token}`;
                }
                return config;
            });
            Api.c.interceptors.response.use(
                (response: AxiosResponse) => {
                    return response.data;
                },
                (error: any) => {
                    if (!error.response) {
                        return Promise.reject(error);
                    }
                    if (error.response.status === 401) {
                        // TODO: logout
                        return Promise.reject(error);
                    }
                    if (error.response.status === 302) {
                        // TODO: handle redirect
                        return Promise.reject(error);
                    }
                    if (error.response.status === 404) {
                        // TODO: redirect to 404 page
                        return Promise.reject(error);
                    }
                    if (error.response.status === 500) {
                        return Promise.reject(error);
                    }
                    const data = error.response.data;
                    const errorData: string = (data && data.message) || "Unknown error";
                    return Promise.reject(errorData);
                },
            );
        }

        return Api.c as Client;
    }

    static setToken(token: string) {
        Api.token = token;
    }
}
