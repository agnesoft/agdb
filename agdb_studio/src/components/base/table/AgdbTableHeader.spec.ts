import { shallowMount } from "@vue/test-utils";
import AgdbTableHeader from "./AgdbTableHeader.vue";
import { describe, beforeEach, vi, it, expect } from "vitest";

describe("TableHeader", () => {
    it("should render", () => {
        const wrapper = shallowMount(AgdbTableHeader, {
            props: {
                tableKey: "table",
            },
        });
        expect(wrapper.exists()).toBe(true);
    });
});
