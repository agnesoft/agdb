import { describe, it, expect, afterEach } from "vitest";
import apiInstance from "@/services/api.service";
import MockAdapter from "axios-mock-adapter";
import type { AxiosInstance } from "axios";
import { readFileSync } from "fs";

const mock = new MockAdapter(apiInstance.client?.api.getAxiosInstance() as AxiosInstance);
const schema = readFileSync("../../agdb_server/openapi/schema.json", "utf8");

describe("apiInstance", () => {
    afterEach(() => {
        mock.reset();
        mock.onGet("/agdb_server/openapi/schema.json").reply(200, schema);
    });

    it("initial test", () => {
        apiInstance.client?.paths["/api/v1/admin/db/list"]
            .get()
            .then((res) => {
                expect(res.status).toBe(401);
            })
            .catch((err) => {});
    });
});
