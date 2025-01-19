import { useAuth, setLocalStorageToken, refreshToken } from "./auth";
import { get_token } from "@/tests/apiMock";
import { ACCESS_TOKEN } from "@/constants";
import { vi, describe, it, beforeEach, expect } from "vitest";

describe("auth service", () => {
    Object.defineProperty(window, "location", {
        value: { reload: vi.fn() },
    });

    const { isLoggedIn, logout, login, token } = useAuth();
    beforeEach(() => {
        localStorage.removeItem(ACCESS_TOKEN);
    });
    describe("isLoggedIn", () => {
        beforeEach(() => {
            localStorage.removeItem(ACCESS_TOKEN);
        });
        it("returns false if no token", () => {
            expect(isLoggedIn.value).toBe(false);
        });
        it("returns true if token", () => {
            get_token.mockReturnValueOnce(undefined);
            setLocalStorageToken("test");
            expect(isLoggedIn.value).toBe(true);
            expect(token.value).toBe("test");
        });
    });
    describe("logout", () => {
        Object.defineProperty(window, "location", {
            value: { reload: vi.fn() },
        });
        beforeEach(() => {
            localStorage.removeItem(ACCESS_TOKEN);
            refreshToken();
        });
        it("does nothing if not logged in", async () => {
            await logout();
            expect(isLoggedIn.value).toBe(false);
        });
        it("logs out if logged in", async () => {
            setLocalStorageToken("test");
            await logout();
            expect(isLoggedIn.value).toBe(false);
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
    describe("setLocalStorageToken", () => {
        it("sets token", () => {
            setLocalStorageToken("test");
            expect(localStorage.getItem(ACCESS_TOKEN)).toBe("test");
        });
    });
    describe("refreshToken", () => {
        beforeEach(() => {
            localStorage.removeItem(ACCESS_TOKEN);
        });
        it("refreshes token", () => {
            setLocalStorageToken("test");
            expect(isLoggedIn.value).toBe(true);
        });
        it("refresh page if no token", () => {
            refreshToken();
            expect(isLoggedIn.value).toBe(false);
            expect(window.location.reload).toHaveBeenCalled();
        });
    });
});
