import { vi, describe, it, beforeEach, expect } from "vitest";
import AdminView from "./AdminView.vue";
import { shallowMount } from "@vue/test-utils";

describe("AdminView", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("renders the admin db view", () => {
        const wrapper = shallowMount(AdminView);
        expect(wrapper.text()).toContain("Admin View");
    });
});
