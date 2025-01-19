import { vi, describe, it, beforeEach, expect } from "vitest";
import AdminUserView from "./AdminUserView.vue";
import { shallowMount } from "@vue/test-utils";

describe("AdminUserView", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("renders the admin user view", () => {
        const wrapper = shallowMount(AdminUserView);
        expect(wrapper.text()).toContain("User View");
    });
});
