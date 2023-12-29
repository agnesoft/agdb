import type { Client } from "../../../agdb_api/typescript/src/schema";
import OpenAPIClientAxios from "openapi-client-axios";

export class Api {
    private static api: OpenAPIClientAxios | undefined = undefined;
    private static c: Client | undefined = undefined;

    static async client(): Promise<Client> {
        if (Api.c === undefined) {
            Api.api = new OpenAPIClientAxios({
                definition: `${import.meta.env.VITE_API_URL}/api/v1/openapi.json`,
            });

            Api.c = await Api.api.init<Client>();
            Api.c.interceptors.response.use(
                (response) => {
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
}
