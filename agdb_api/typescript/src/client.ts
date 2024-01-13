import { OpenAPIClientAxios } from "openapi-client-axios";
import type { Client } from "./schema";

export class AgdbApi {
    private static api: OpenAPIClientAxios | undefined = undefined;
    private static c: Client | undefined = undefined;
    private static token: string = "";

    static async client(host: String, port: number): Promise<Client> {
        if (AgdbApi.c === undefined) {
            AgdbApi.api = new OpenAPIClientAxios({
                definition: `${host}:${port}/api/v1/openapi.json`,
            });
            AgdbApi.c = await AgdbApi.api.init<Client>();
            AgdbApi.c.interceptors.request.use((config) => {
                if (AgdbApi.token !== "") {
                    config.headers.Authorization = `Bearer ${AgdbApi.token}`;
                }
                return config;
            });
        }

        return AgdbApi.c as Client;
    }

    static setToken(token: string) {
        AgdbApi.token = token;
    }
}
