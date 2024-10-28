import { describe, it, expect, beforeEach } from "vitest";
import { createRouter, createWebHistory } from "vue-router";
import { mount } from "@vue/test-utils";
import { createRoutes } from "@/router/routes";
import HomeView from "@/views/HomeView.vue";
import AboutView from "@/views/HomeView.vue";
import LoginView from "@/views/LoginView.vue";
import App from "@/App.vue";

const setupTest = async (path: string) => {
    const router = createRouter({
        history: createWebHistory(),
        routes: createRoutes(),
    });
    router.push(path);
    await router.isReady();
    const wrapper = mount(App, {
        global: {
            plugins: [router],
        },
    });
    return { router, wrapper };
};

describe("router", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("renders a default route", async () => {
        const { wrapper } = await setupTest("/");

        expect(wrapper.findComponent(HomeView).exists()).toBe(true);
    });

    it("renders the about route", async () => {
        const { wrapper, router } = await setupTest("/");
        router.push("/about");
        await router.isReady();

        expect(wrapper.findComponent(AboutView).exists()).toBe(true);
    });

    it("renders the login route", async () => {
        const { wrapper } = await setupTest("/login");

        expect(wrapper.findComponent(LoginView).exists()).toBe(true);
    });
});
