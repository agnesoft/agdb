import { AgdbApi, AgdbApiError } from "../src/api.ts";
import { FetchClient } from "../src/fetch_client.ts";
import { QueryBuilder } from "../src/index.ts";
import { describe, expect, it } from "vitest";

const ADDRESS = "localhost:3000";

async function admin(): Promise<AgdbApi<FetchClient>> {
    const api = AgdbApi.new(new FetchClient(), ADDRESS);
    await api.user_login({ username: "admin", password: "admin" });
    return api;
}

async function user(name: string): Promise<AgdbApi<FetchClient>> {
    await (
        await admin()
    ).admin_user_add({ username: name, password: "password123" });
    const api = AgdbApi.new(new FetchClient("fetch_client_test"), ADDRESS);
    await api.user_login({ username: name, password: "password123" });
    return api;
}

describe("FetchClient", () => {
    it("gets without a token", async () => {
        const api = AgdbApi.new(new FetchClient(), `http://${ADDRESS}`);
        expect(await api.status()).toBe(200);
    });

    it("posts the credentials and keeps the token", async () => {
        const api = await admin();
        expect(api.token).toBeTypeOf("string");
        const [status, dbs] = await api.db_list();
        expect(status).toBe(200);
        expect(Array.isArray(dbs)).toBe(true);
    });

    it("throws the status and body of a failure", async () => {
        const api = AgdbApi.new(new FetchClient(), ADDRESS);
        const error = await api.db_list().catch((e: unknown) => e);
        expect(error).toBeInstanceOf(AgdbApiError);
        expect((error as AgdbApiError).status).toBe(401);
        expect((error as AgdbApiError).to_string()).toMatch(/^401: /);
    });

    it("throws status 500 when the request fails", async () => {
        const api = AgdbApi.new(new FetchClient(), "localhost:1");
        await expect(api.status()).rejects.toMatchObject({ status: 500 });
    });

    it("executes queries, puts and deletes", async () => {
        const api = await user("fetch_client_user");
        expect(
            await api.db_add({
                owner: "fetch_client_user",
                db: "db",
                db_type: "memory",
            }),
        ).toBe(201);

        const [status, results] = await api.db_exec_mut({
            owner: "fetch_client_user",
            db: "db",
            queries: [QueryBuilder.insert().nodes().count(2).query()],
        });
        expect(status).toBe(200);
        expect(results[0].result).toBe(2);

        const owner = "fetch_client_user";
        expect(
            await api.dashboard_add({ owner, name: "ops", config: "{}" }),
        ).toBe(201);
        expect(
            await api.dashboard_save({ owner, name: "ops", config: '{"v":2}' }),
        ).toBe(201);
        expect(
            (await api.dashboard_get({ owner, name: "ops" }))[1].config,
        ).toBe('{"v":2}');
        // No home is `null` on the wire, `undefined` as the `Option` type says.
        expect((await api.user_home_dashboard())[1]).toBeUndefined();
        expect(await api.user_home_dashboard_set({ owner, name: "ops" })).toBe(
            201,
        );
        expect((await api.user_home_dashboard())[1]).toEqual({
            owner,
            name: "ops",
        });
        expect(await api.user_home_dashboard_clear()).toBe(204);
        expect(await api.dashboard_remove({ owner, name: "ops" })).toBe(204);
        expect(
            await api.db_delete({ owner: "fetch_client_user", db: "db" }),
        ).toBe(204);
    });

    it("logs out and forgets the token", async () => {
        const api = await admin();
        expect(await api.user_logout()).toBe(201);
        expect(api.token).toBeUndefined();
    });
});
