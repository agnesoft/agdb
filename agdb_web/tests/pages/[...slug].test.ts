import { shallowMount } from "@vue/test-utils";
import { describe, it, expect, vi, beforeEach } from "vitest";
import slug from "@/pages/[...slug].vue";

describe("[...slug]", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("renders content", () => {
        const wrapper = shallowMount(slug, {
            route: "/about",
            global: {
                plugins: [useRouter()],
                stubs: {
                    "nuxt-layout": {
                        template: "<div><slot /></div>",
                    },
                },
            },
        });
        expect(wrapper.getComponent("content-doc-stub")).toBeTruthy();
    });

    it("renders content for a specific locale", () => {
        const wrapper = shallowMount(slug, {
            route: "/cs/o-agdb",
            global: {
                plugins: [useRouter()],
                stubs: {
                    "nuxt-layout": {
                        template: "<div><slot /></div>",
                    },
                },
            },
        });
        expect(wrapper.getComponent("content-doc-stub")).toBeTruthy();
    });
});
