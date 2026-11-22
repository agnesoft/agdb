import { AgdbApiError, HttpClient } from "./api.js";
import type { AgdbApiClient } from "./api.js";

/**
 * The transport of the generated `AgdbApi` over the native `fetch`: bodies
 * are JSON and the token is a bearer one. A response that is not a success
 * and a failed request throw an `AgdbApiError`, the latter with status 500.
 */
export class FetchClient extends HttpClient implements AgdbApiClient {
    constructor(public user_agent: string = "agdb_api") {
        super();
    }

    async delete(uri: string, token: string | undefined): Promise<number> {
        return (await this.send("DELETE", uri, undefined, token)).status;
    }

    async get<T>(uri: string, token: string | undefined): Promise<[number, T]> {
        const response = await this.send("GET", uri, undefined, token);
        return [response.status, await body<T>(response)];
    }

    async post<T, R>(
        uri: string,
        json: T | undefined,
        token: string | undefined,
    ): Promise<[number, R]> {
        const response = await this.send("POST", uri, json, token);
        return [response.status, await body<R>(response)];
    }

    async put<T>(
        uri: string,
        json: T | undefined,
        token: string | undefined,
    ): Promise<number> {
        return (await this.send("PUT", uri, json, token)).status;
    }

    private async send(
        method: string,
        uri: string,
        json: unknown,
        token: string | undefined,
    ): Promise<Response> {
        const headers: Record<string, string> = {
            "User-Agent": this.user_agent,
        };

        if (token !== undefined) {
            headers["Authorization"] = `Bearer ${token}`;
        }

        if (json !== undefined) {
            headers["Content-Type"] = "application/json";
        }

        const response = await fetch(uri, {
            method,
            headers,
            body: json === undefined ? undefined : JSON.stringify(json),
        }).catch((e: unknown) => {
            throw new AgdbApiError(500, String(e));
        });

        if (!response.ok) {
            throw new AgdbApiError(response.status, await response.text());
        }

        return response;
    }
}

/**
 * The JSON of the body, `undefined` when it is empty or `null`: the generated
 * client types a Rust `Option` as `T | undefined`.
 */
async function body<T>(response: Response): Promise<T> {
    const text = await response.text();
    const json: unknown = text === "" ? undefined : JSON.parse(text);
    return (json === null ? undefined : json) as T;
}
