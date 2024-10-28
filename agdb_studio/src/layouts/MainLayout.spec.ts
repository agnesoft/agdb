import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import MainLayout from "@/layouts/MainLayout.vue";
import { createRouter, createWebHistory } from "vue-router";
const { loginMock, isLoggedInMock, logoutMock } = vi.hoisted(() => {
    return {
        loginMock: vi.fn(),
        isLoggedInMock: vi.fn(),
        logoutMock: vi.fn(),
    };
});
vi.mock("@/services/auth.service", () => {
    return {
        login: loginMock,
        isLoggedIn: isLoggedInMock,
        logout: logoutMock,
    };
});
const routes = [
    { path: "/", name: "home", component: { template: "<div>Home</div>" } },
    {
        path: "/about",
        name: "about",
        component: { template: "<div>About</div>" },
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
        ).toContain("Home");

        // about page link exists
        expect(
            links.find((link) => link.props("to") === "/about")?.text(),
        ).toContain("About");
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

    it("logout on click", async () => {
        const wrapper = mount(MainLayout, {
            global: {
                plugins: [router],
            },
        });

        await wrapper.find(".logout-button").trigger("click");

        expect(logoutMock).toHaveBeenCalled();
    });
});