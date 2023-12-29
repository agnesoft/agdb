import OpenAPIClientAxios from "openapi-client-axios";
import type { Client } from "@/openapi/schema";

class Api {
    public host: string = process.env.VUE_APP_API_URL as string;
    public api?: OpenAPIClientAxios;
    public client?: Client;

    static async create(): Promise<Api> {
        const api = new Api();
        api.api = new OpenAPIClientAxios({
            definition: `${api.host}/openapi/schema.json`,
        });
        console.log(`HOST: ${api.host}`);
        api.client = await api.api.init<Client>();
        api.client.interceptors.response.use(
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
        return api;
    }
}

const apiInstance = await Api.create();

export default apiInstance;
