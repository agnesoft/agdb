import type { Client } from "../src/schema";
import OpenAPIClientAxios from "openapi-client-axios";

export class Api {
    private static api: OpenAPIClientAxios | undefined = undefined;
    private static c: Client | undefined = undefined;
    private static token: string = "";

    static async client(): Promise<Client> {
        if (Api.c === undefined) {
            Api.api = new OpenAPIClientAxios({
                definition: `http://localhost:3000/api/v1/openapi.json`,
            });
            Api.c.interceptors.request.use((config) => {
                if (Api.token !== "") {
                    config.headers.Authorization = `Bearer ${Api.token}`;
                }
                return config;
            });
            Api.c = await Api.api.init<Client>();
        }

        return Api.c as Client;
    }

    static setToken(token: string) {
        Api.token = token;
    }
}
