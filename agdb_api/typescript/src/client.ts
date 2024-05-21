import { OpenAPIClientAxios } from "openapi-client-axios";
import type { Client } from "./schema";

export class AgdbApi {
    private static api: OpenAPIClientAxios | undefined = undefined;
    private static c: Client | undefined = undefined;
    private static token: string = "";

    static async client(
        host: String,
        port: number,
        publicPath?: string,
    ): Promise<Client> {
        if (AgdbApi.c === undefined) {
            try {
                let baseurl_tmp = `${host}:${port}`;
                if (typeof publicPath !== "undefined") {
                    baseurl_tmp = baseurl_tmp + `/${publicPath}`;
                }
                const baseURL = baseurl_tmp;
                AgdbApi.api = new OpenAPIClientAxios({
                    definition: `/api/v1/openapi.json`,
                    axiosConfigDefaults: {
                        baseURL: baseURL, // set axios baseURL
                    },
                });
                AgdbApi.c = await AgdbApi.api.init<Client>();
                AgdbApi.c.interceptors.request.use((config) => {
                    if (AgdbApi.token !== "") {
                        config.headers.Authorization = `Bearer ${AgdbApi.token}`;
                    }
                    return config;
                });
            } catch (e) {
                console.log(e);
            }
        }

        return AgdbApi.c as Client;
    }

    static setToken(token: string) {
        AgdbApi.token = token;
    }
}
