import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import MainLayout from "./MainLayout.vue";
import { createRouter, createWebHistory } from "vue-router";
import { ref } from "vue";

const { loginMock, isLoggedInMock, logoutMock } = vi.hoisted(() => {
  return {
    loginMock: vi.fn(),
    isLoggedInMock: { value: true },
    logoutMock: vi.fn(),
    isAdminView: { value: false },
  };
});

vi.mock("@agdb-studio/auth/src/auth", () => {
  return {
    useAuth: () => ({
      login: loginMock,
      isLoggedIn: isLoggedInMock,
      logout: logoutMock,
      token: { value: "test" },
    }),
  };
});

const isAdminView = ref(false);

vi.mock("@agdb-studio/profile/src/composables/admin", () => {
  return {
    useAdmin: () => ({
      isAdminView,
      isAdmin: { value: true },
    }),
  };
});

const routes = [
  { path: "/", name: "home", component: { template: "<div>Home</div>" } },
  {
    path: "/db",
    name: "db",
    component: { template: "<div>Databases</div>" },
  },
  {
    path: "/admin",
    name: "admin",
    children: [
      {
        path: "users",
        name: "admin-users",
        component: { template: "<div>Admin Users</div>" },
      },
      {
        path: "db",
        name: "admin-db",
        component: { template: "<div>Admin DB</div>" },
      },
    ],
  },
];

const router = createRouter({
  history: createWebHistory("/studio"),
  routes,
});

describe("MainLayout", () => {
  it("renders user navigation links", async () => {
    isAdminView.value = false;
    const wrapper = mount(MainLayout, {
      global: {
        plugins: [router],
      },
    });
    await router.push("/");

    const links = wrapper.findAllComponents({ name: "RouterLink" });
    expect(links.length).toBeGreaterThan(1);

    // home page link exists
    expect(
      links.find((link) => link.props("to") === "/")?.text(),
    ).toBeDefined();

    // db page link exists
    expect(links.find((link) => link.props("to") === "/db")?.text()).toContain(
      "Databases",
    );

    expect(wrapper.find(".admin-label").exists()).toBe(false);
  });

  it("renders the router view", async () => {
    isAdminView.value = false;
    const wrapper = mount(MainLayout, {
      global: {
        plugins: [router],
      },
    });

    await router.push("/db");
    await router.isReady();

    expect(wrapper.text()).toContain("Databases");
  });

  it("renders admin navigation links", async () => {
    isAdminView.value = true;
    const wrapper = mount(MainLayout, {
      global: {
        plugins: [router],
      },
    });

    await router.push("/admin");
    await router.isReady();

    const links = wrapper.findAllComponents({ name: "RouterLink" });
    expect(links.length).toBeGreaterThan(1);

    // admin users page link exists
    expect(
      links.find((link) => link.props("to") === "/admin/users")?.text(),
    ).toContain("Users");

    // admin db page link exists
    expect(
      links.find((link) => link.props("to") === "/admin/db")?.text(),
    ).toContain("Databases");

    expect(wrapper.find(".admin-label").exists()).toBe(true);
  });
});
