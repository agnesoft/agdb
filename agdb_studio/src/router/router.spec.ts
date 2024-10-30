import { describe, it, expect, beforeEach } from "vitest";
import router from "@/router/router";

const { isLoggedInMock, logoutMock } = vi.hoisted(() => {
    return { isLoggedInMock: vi.fn(), logoutMock: vi.fn() };
});

vi.mock("@/services/auth.service", () => {
    return {
        isLoggedIn: isLoggedInMock,
        logout: logoutMock,
    };
});

describe("router", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("redirects to login if not logged in", async () => {
        isLoggedInMock.mockReturnValue(false);

        await router.push("/");

        expect(router.currentRoute.value.name).toBe("login");
    });
    it("navigates to home if logged in", async () => {
        isLoggedInMock.mockReturnValue(true);

        await router.push("/");

        expect(router.currentRoute.value.name).toBe("home");
    });
    it("logout if logged in and navigates to login", async () => {
        isLoggedInMock.mockReturnValue(true);

        await router.push("/login");

        expect(router.currentRoute.value.name).toBe("login");
        expect(logoutMock).toHaveBeenCalled();
    });
    it("loads the about page", async () => {
        isLoggedInMock.mockReturnValue(true);

        await router.push("/about");

        expect(router.currentRoute.value.name).toBe("about");
    });
});
