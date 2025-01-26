import { describe, it, expect, beforeEach, vi } from "vitest";
import router from "@/router/router";

const { isLoggedInMock, logoutMock, admin, fetchUserStatus } = vi.hoisted(
    () => {
        return {
            isLoggedInMock: { value: true },
            logoutMock: vi.fn(),
            admin: { value: true },
            fetchUserStatus: vi.fn(),
        };
    },
);

vi.mock("@/composables/profile/auth", () => {
    return {
        useAuth: () => ({
            isLoggedIn: isLoggedInMock,
            logout: logoutMock,
            token: { value: "test" },
        }),
    };
});

vi.mock("@/composables/profile/account", () => {
    return {
        useAccount: () => ({
            admin,
            fetchUserStatus,
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

    it("redirects to home if user is not admin", async () => {
        admin.value = false;

        await router.push("/admin");

        expect(router.currentRoute.value.name).toBe("home");
    });
    it("loads the admin page if user is admin", async () => {
        admin.value = true;

        await router.push("/admin");

        expect(router.currentRoute.value.name).toBe("admin");
    });
    it("loads the admin db page if user is admin", async () => {
        admin.value = true;

        await router.push("/admin/db");

        expect(router.currentRoute.value.name).toBe("admin-db");
    });

    it("loads the admin users page if user is admin", async () => {
        admin.value = true;

        await router.push("/admin/users");

        expect(router.currentRoute.value.name).toBe("admin-users");
    });
});
