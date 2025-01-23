import { vi, describe, it, beforeEach, expect } from "vitest";
import AdminDbView from "./AdminDbView.vue";
import { shallowMount } from "@vue/test-utils";

describe("AdminDbView", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("renders the admin db view", () => {
        const wrapper = shallowMount(AdminDbView);
        expect(wrapper.text()).toContain("db-view");
    });
});
