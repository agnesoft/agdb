import { shallowMount } from "@vue/test-utils";
import { describe, it, expect } from "vitest";
import slug from "@/pages/[...slug].vue";

describe("[...slug]", () => {
    it("renders the correct message", () => {
        const wrapper = shallowMount(slug, { route: "/" });
        expect(wrapper.text()).toContain("Hello World");
    });
});
