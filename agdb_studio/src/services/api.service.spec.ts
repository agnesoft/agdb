import {
    getClient,
    initClient,
    responseInterceptor,
    errorInterceptor,
} from "./api.service";
import { client } from "@/tests/authMock";
import type { AxiosError, AxiosResponse } from "axios";

describe("client service", () => {
    Object.defineProperty(window, "location", {
        value: { reload: vi.fn() },
    });

    describe("getClient", () => {
        it("returns client", () => {
            expect(getClient()).toBeDefined();
        });
    });
    describe("initClient", () => {
        it("catches axios errors", async () => {
            client.mockRejectedValueOnce("error");
            await initClient().catch((error) => {
                expect(error).toBe("error");
            });
        });
    });
    describe("responseInterceptor", () => {
        it("returns response", () => {
            const response = {
                data: "response",
                status: 200,
                statusText: "OK",
            };
            expect(
                responseInterceptor(
                    response as unknown as AxiosResponse<string>,
                ),
            ).toBe(response);
        });
    });
    describe("errorInterceptor", () => {
        it("returns error for 401 response", () => {
            const response = {
                message: "error",
                response: { status: 401 },
            };
            expect(
                errorInterceptor(response as unknown as AxiosError<string>),
            ).rejects.toBe(response);
        });
        it("returns error for unknown response", () => {
            const response = {
                message: "error",
            };
            expect(
                errorInterceptor(response as unknown as AxiosError<string>),
            ).rejects.toBe(response);
        });
    });
});
