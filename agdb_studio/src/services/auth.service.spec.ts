import type { AxiosError, AxiosResponse } from "axios";
import {
    isLoggedIn,
    logout,
    login,
    ACCESS_TOKEN,
    getClient,
    initClient,
    responseInterceptor,
    errorInterceptor,
} from "./auth.service";
import { client, get_token } from "@/tests/authMock";

describe("auth service", () => {
    beforeEach(() => {
        localStorage.removeItem(ACCESS_TOKEN);
    });
    describe("isLoggedIn", () => {
        beforeEach(() => {
            localStorage.removeItem(ACCESS_TOKEN);
        });
        it("returns false if no token", () => {
            expect(isLoggedIn()).toBe(false);
        });
        it("returns true if token", () => {
            get_token.mockReturnValueOnce(undefined);
            localStorage.setItem(ACCESS_TOKEN, "test");
            expect(isLoggedIn()).toBe(true);
        });
    });
    describe("logout", () => {
        Object.defineProperty(window, "location", {
            value: { reload: vi.fn() },
        });
        beforeEach(() => {
            localStorage.removeItem(ACCESS_TOKEN);
        });
        it("does nothing if not logged in", async () => {
            await logout();
            expect(isLoggedIn()).toBe(false);
        });
        it("logs out if logged in", async () => {
            localStorage.setItem(ACCESS_TOKEN, "test");
            await logout();
            expect(isLoggedIn()).toBe(false);
        });
    });
    describe("login", () => {
        it("returns token on success", async () => {
            login("test", "test").then((token) => {
                expect(token).toBe("token");
            });
        });
        it("throws error on failure", async () => {
            login("test", "test").catch((error) => {
                expect(error).toBe("error");
            });
        });
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
