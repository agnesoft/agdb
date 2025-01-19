import { describe, it, expect, beforeEach, vi } from "vitest";
import router from "@/router/router";

const { isLoggedInMock, logoutMock } = vi.hoisted(() => {
    return { isLoggedInMock: { value: true }, logoutMock: vi.fn() };
});

vi.mock("@/composables/user/auth", () => {
    return {
        useAuth: () => ({
            isLoggedIn: isLoggedInMock,
            logout: logoutMock,
            token: { value: "test" },
        }),
    };
});

describe("router", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("redirects to login if not logged in", async () => {
        isLoggedInMock.value = false;

        await router.push("/");

        expect(router.currentRoute.value.name).toBe("login");
    });
    it("navigates to home if logged in", async () => {
        isLoggedInMock.value = true;

        await router.push("/");

        expect(router.currentRoute.value.name).toBe("home");
    });
    it("redirects to home if logged user tries to access login page", async () => {
        isLoggedInMock.value = true;

        await router.push("/login");

        expect(router.currentRoute.value.name).toBe("home");
    });
    it("loads the databases page", async () => {
        isLoggedInMock.value = true;

        await router.push("/db");

        expect(router.currentRoute.value.name).toBe("db");
    });
});
