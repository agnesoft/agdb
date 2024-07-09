import { OpenAPIClientAxios } from "openapi-client-axios";
import type { Client } from "./schema";

export class AgdbApi {
    private static api: OpenAPIClientAxios | undefined = undefined;
    private static c: Client | undefined = undefined;
    private static token: string = "";

    static async client(address: String): Promise<Client> {
        if (AgdbApi.c === undefined) {
            AgdbApi.api = new OpenAPIClientAxios({
                definition: `${address}/api/v1/openapi.json`,
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

    static async login(username: string, password: string): Promise<void> {
        if (AgdbApi.c === undefined) {
            throw new Error("client not initialized");
        }

        let token = await AgdbApi.c.user_login(null, {
            username: username,
            password: password,
        });

        AgdbApi.setToken(token.data);
    }

    static setToken(token: string) {
        AgdbApi.token = token;
    }
}
