import { isLoggedIn, logout, login } from "./auth.service";
import { get_token } from "@/tests/authMock";
import { ACCESS_TOKEN } from "@/constants";

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
});
