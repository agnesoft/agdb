import { OpenAPIClientAxios } from "openapi-client-axios";
import type { Client } from "./schema";

export type AgdbApiClient = Client & AgdbApi;

export class AgdbApi {
    private c: AgdbApiClient;

    static async client(address: String): Promise<AgdbApiClient> {
        return await new AgdbApi().create(address);
    }

    async create(address: String): Promise<AgdbApiClient> {
        let api: OpenAPIClientAxios = new OpenAPIClientAxios({
            definition: `${address}/api/v1/openapi.json`,
        });
        let client = (await api.init<Client>()) as AgdbApiClient;
        client.c = client;
        client.login = this.login;
        client.set_token = this.set_token;

        return client;
    }

    async login(username: string, password: string): Promise<void> {
        let token = await this.c.user_login(null, {
            username: username,
            password: password,
        });

        this.set_token(token.data);
    }

    async set_token(token: string) {
        this.c.interceptors.request.use((config) => {
            config.headers.Authorization = `Bearer ${token}`;
            return config;
        });
    }
}
