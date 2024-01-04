import { shallowMount } from "@vue/test-utils";
import { describe, it, expect } from "vitest";
import { createI18n } from "vue-i18n";
import { useRouter } from "nuxt/app";
import i18nConfig from "@/i18n.config";
import slug from "@/pages/[...slug].vue";

describe("[...slug]", () => {
    it("renders the correct message", () => {
        const i18n = createI18n(i18nConfig);
        const wrapper = shallowMount(slug, {
            route: "/about",
            global: {
                plugins: [i18n, useRouter()],
            },
        });
        expect(wrapper.text()).toContain("agdb");
    });
});
