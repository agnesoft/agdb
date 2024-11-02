import { describe, it, expect, beforeEach } from "vitest";
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
    it("logout if logged in and navigates to login", async () => {
        isLoggedInMock.value = true;

        await router.push("/login");

        expect(router.currentRoute.value.name).toBe("login");
        expect(logoutMock).toHaveBeenCalled();
    });
    it("loads the about page", async () => {
        isLoggedInMock.value = true;

        await router.push("/about");

        expect(router.currentRoute.value.name).toBe("about");
    });
});
