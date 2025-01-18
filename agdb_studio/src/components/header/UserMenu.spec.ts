import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import useModal from "@/composables/modal/modal";
import UserMenu from "./UserMenu.vue";
import { user_change_password } from "@/tests/apiMock";

const { logout } = vi.hoisted(() => {
    return {
        logout: vi.fn(),
    };
});

vi.mock("@/composables/user/auth", () => {
    return {
        useAuth: () => ({
            logout,
        }),
    };
});

vi.mock("@/composables/user/account", () => {
    return {
        useAccount: () => ({
            username: "testUser",
        }),
    };
});

const { modalIsVisible, closeModal, handleConfirm } = useModal();

describe("UserDropdown", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        closeModal();
    });
    it("renders the user actions", () => {
        const wrapper = mount(UserMenu);
        expect(wrapper.text()).toContain("Change password");
        expect(wrapper.text()).toContain("Logout");
    });

    it("should logout on click", async () => {
        const wrapper = mount(UserMenu);
        const logoutElement = wrapper.find(".menu-item[data-key=logout]");
        logoutElement.trigger("click");
        await wrapper.vm.$nextTick();
        expect(modalIsVisible.value).toBe(true);
        handleConfirm();
        await wrapper.vm.$nextTick();
        expect(logout).toHaveBeenCalled();
    });

    it("should open the change password modal on click", async () => {
        const wrapper = mount(UserMenu);
        const changePasswordElement = wrapper.find(
            ".menu-item[data-key=change-password]",
        );
        changePasswordElement.trigger("click");
        expect(modalIsVisible.value).toBe(false);
        await wrapper.vm.$nextTick();
        expect(modalIsVisible.value).toBe(true);
    });

    it("should change password on confirm", async () => {
        const wrapper = mount(UserMenu);
        const changePasswordElement = wrapper.find(
            ".menu-item[data-key=change-password]",
        );
        changePasswordElement.trigger("click");
        await wrapper.vm.$nextTick();
        expect(modalIsVisible.value).toBe(true);
        handleConfirm();
        await wrapper.vm.$nextTick();
        expect(modalIsVisible.value).toBe(false);
        expect(user_change_password).toHaveBeenCalled();
    });
});
