import { shallowMount } from "@vue/test-utils";
import { describe, it, expect } from "vitest";
import app from "@/app.vue";

describe("app", () => {
    it("renders the correct message", () => {
        const wrapper = shallowMount(app, { route: "/" });
        expect(wrapper.html()).toContain("<nuxt-page-stub>");
    });
});
