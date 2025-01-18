import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import MainLayout from "./MainLayout.vue";
import { createRouter, createWebHistory } from "vue-router";

const { loginMock, isLoggedInMock, logoutMock } = vi.hoisted(() => {
    return {
        loginMock: vi.fn(),
        isLoggedInMock: { value: true },
        logoutMock: vi.fn(),
    };
});

vi.mock("@/composables/user/auth", () => {
    return {
        useAuth: () => ({
            login: loginMock,
            isLoggedIn: isLoggedInMock,
            logout: logoutMock,
            token: { value: "test" },
        }),
    };
});

const routes = [
    { path: "/", name: "home", component: { template: "<div>Home</div>" } },
    {
        path: "/about",
        name: "about",
        component: { template: "<div>About</div>" },
    },
    {
        path: "/db",
        name: "db",
        component: { template: "<div>Databases</div>" },
    },
];

const router = createRouter({
    history: createWebHistory(),
    routes,
});

describe("MainLayout", () => {
    it("renders navigation links", () => {
        const wrapper = mount(MainLayout, {
            global: {
                plugins: [router],
            },
        });

        const links = wrapper.findAllComponents({ name: "RouterLink" });
        expect(links.length).toBeGreaterThan(1);

        // home page link exists
        expect(
            links.find((link) => link.props("to") === "/")?.text(),
        ).toBeDefined();

        // about page link exists
        expect(
            links.find((link) => link.props("to") === "/db")?.text(),
        ).toContain("Databases");
    });

    it("renders the router view", async () => {
        const wrapper = mount(MainLayout, {
            global: {
                plugins: [router],
            },
        });

        await router.push("/about");
        await router.isReady();

        expect(wrapper.text()).toContain("About");
    });
});
