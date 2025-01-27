import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import LoginForm from "@/components/auth/LoginForm.vue";

const { loginMock, logoutMock, pushMock, currentRoute } = vi.hoisted(() => {
    return {
        loginMock: vi.fn(),
        logoutMock: vi.fn(),
        pushMock: vi.fn(),
        currentRoute: {
            value: {
                query: {
                    redirect: "/home",
                } as { redirect?: string },
            },
        },
    };
});

vi.mock("@/composables/profile/auth", () => {
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
            currentRoute,
        },
    };
});

describe("LoginForm", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("runs successful login on click and redirects from query", async () => {
        loginMock.mockResolvedValue(true);

        const wrapper = mount(LoginForm);
        currentRoute.value.query.redirect = "/home";
        await wrapper.find('input[type="text"]#username').setValue("test");
        await wrapper.find('input[type="password"]#password').setValue("test");
        await wrapper.find("input[type='checkbox']").setValue(false);

        await wrapper.find(".login-form>form").trigger("submit");

        expect(loginMock).toHaveBeenCalledWith({
            username: "test",
            password: "test",
            cluster: false,
        });
        expect(pushMock).toHaveBeenCalledWith("/home");
    });
    it("runs successful cluster login on click and redirects from query", async () => {
        loginMock.mockResolvedValue(true);

        const wrapper = mount(LoginForm);
        currentRoute.value.query.redirect = "/home";
        await wrapper.find('input[type="text"]#username').setValue("test");
        await wrapper.find('input[type="password"]#password').setValue("test");
        await wrapper.find("input[type='checkbox']").setValue(true);

        await wrapper.find(".login-form>form").trigger("submit");

        expect(loginMock).toHaveBeenCalledWith({
            username: "test",
            password: "test",
            cluster: true,
        });
        expect(pushMock).toHaveBeenCalledWith("/home");
    });
    it("runs successful login on click and redirects to home", async () => {
        loginMock.mockResolvedValue(true);

        const wrapper = mount(LoginForm);
        currentRoute.value.query.redirect = undefined;
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
