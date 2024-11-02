import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import LoginForm from "@/components/auth/LoginForm.vue";
import { useAuth } from "@/composables/user/auth";

const { loginMock, logoutMock, pushMock } = vi.hoisted(() => {
    return {
        loginMock: vi.fn(),
        logoutMock: vi.fn(),
        pushMock: vi.fn(),
    };
});

vi.mock("@/composables/user/auth", () => {
    return {
        useAuth: () => ({
            login: loginMock,
            logout: logoutMock,
        }),
    };
});
vi.mock("@/router", () => {
    return {
        default: {
            push: pushMock,
        },
    };
});

describe("LoginForm", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("runs successful login on click", async () => {
        loginMock.mockResolvedValue(true);

        const wrapper = mount(LoginForm);
        await wrapper.find('input[type="text"]#username').setValue("test");
        await wrapper.find('input[type="password"]#password').setValue("test");

        await wrapper.find(".login-form>form").trigger("submit");

        expect(loginMock).toHaveBeenCalled();
        expect(pushMock).toHaveBeenCalledWith({ name: "home" });
    });
    it("runs failed login on click", async () => {
        loginMock.mockRejectedValue("error");

        const wrapper = mount(LoginForm);
        await wrapper.find('input[type="text"]#username').setValue("test");
        await wrapper.find('input[type="password"]#password').setValue("test");

        await wrapper.find(".login-form>form").trigger("submit");

        expect(loginMock).toHaveBeenCalled();
        expect(pushMock).not.toHaveBeenCalled();
    });
});
