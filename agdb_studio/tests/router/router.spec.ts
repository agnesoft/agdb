import { describe, it, expect, beforeEach } from "vitest";
import { createRouter, createWebHistory } from "vue-router";
import { mount } from "@vue/test-utils";
import { routes } from "../../src/router/index";
import type { Router } from "vue-router";
import HomeView from "../../src/views/HomeView.vue";
import AboutView from "../../src/views/HomeView.vue";
import App from "../../src/App.vue";

let router: Router | undefined;

describe("router", () => {
    beforeEach(async () => {
        router = createRouter({
            history: createWebHistory(),
            routes,
        });

        router.push("/");
        await router.isReady();
    });

    it("renders a default route", async () => {
        if (!router) {
            return;
        }

        const wrapper = mount(App, {
            global: {
                plugins: [router],
                mocks: {
                    $route: { path: "/" },
                },
            },
        });

        expect(wrapper.findComponent(HomeView).exists()).toBe(true);
    });

    it("renders the about route", async () => {
        if (!router) {
            return;
        }

        const wrapper = mount(App, {
            global: {
                plugins: [router],
                mocks: {
                    $route: { path: "/about" },
                },
            },
        });

        expect(wrapper.findComponent(AboutView).exists()).toBe(true);
    });
});
