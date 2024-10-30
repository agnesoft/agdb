import { useAccountStore } from "./account";
import { setActivePinia, createPinia } from "pinia";

const { loginMock, isLoggedInMock, logoutMock } = vi.hoisted(() => {
    return {
        loginMock: vi.fn(),
        isLoggedInMock: vi.fn(),
        logoutMock: vi.fn(),
    };
});

vi.mock("@/services/auth.service", () => {
    return {
        login: loginMock,
        isLoggedIn: isLoggedInMock,
        logout: logoutMock,
    };
});

describe("account store", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        setActivePinia(createPinia());
    });

    it("login", async () => {
        loginMock.mockResolvedValue("token");
        const accountStore = useAccountStore();

        const token = await accountStore.login("test", "test");

        expect(token).toBe("token");
    });

    it("logout", async () => {
        logoutMock.mockResolvedValue(true);
        const accountStore = useAccountStore();

        const result = await accountStore.logout();

        expect(result).toBe(true);
    });

    it("loggedIn true", () => {
        isLoggedInMock.mockReturnValue(true);
        const accountStore = useAccountStore();

        expect(accountStore.loggedIn).toBe(true);
    });
    it("loggedIn false", () => {
        isLoggedInMock.mockReturnValue(false);
        const accountStore = useAccountStore();

        expect(accountStore.loggedIn).toBe(false);
    });
});
