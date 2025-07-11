import { describe, it, expect, beforeEach, vi } from "vitest";
import { clearRouter, createRouter, getRouter } from "./router";
import { createWebHistory, type Router } from "vue-router";

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

vi.mock("@agdb-studio/auth/src/auth", () => {
  return {
    useAuth: () => ({
      isLoggedIn: isLoggedInMock,
      logout: logoutMock,
      token: { value: "test" },
    }),
  };
});

vi.mock("@agdb-studio/auth/src/account", () => {
  return {
    useAccount: () => ({
      admin,
      fetchUserStatus,
    }),
  };
});

describe("router", () => {
  let router: Router;
  beforeEach(() => {
    vi.clearAllMocks();
    createRouter({
      history: createWebHistory("test"),
      routes: [
        {
          name: "home",
          path: "/",
          component: { template: "<div>Home</div>" },
        },
        {
          name: "login",
          path: "/login",
          component: { template: "<div>Login</div>" },
        },
        {
          name: "db",
          path: "/db",
          component: { template: "<div>DB</div>" },
        },

        {
          path: "/admin",
          meta: { requiresAdmin: true, admin: true },
          children: [
            {
              path: "",
              name: "admin",
              component: { template: "<div>Admin Home</div>" },
            },
            {
              path: "db",
              name: "admin-db",
              component: { template: "<div>Admin DB</div>" },
            },
          ],
        },
      ],
    });
    router = getRouter();
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

  // it("loads the not found page", async () => {
  //   isLoggedInMock.value = true;

  //   await router.push("/not-found");

  //   expect(router.currentRoute.value.name).toBe("not-found");
  // });
  describe("getRouter", () => {
    it("throws an error if router is not created", () => {
      clearRouter();
      expect(() => getRouter()).toThrow(
        "Router not created yet. Call createRouter first.",
      );
    });
  });
});
