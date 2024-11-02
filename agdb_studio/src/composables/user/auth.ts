import { getClient, removeToken } from "@/services/api.service";
import { ACCESS_TOKEN } from "@/constants";
import { computed, ref } from "vue";

const accessToken = ref<string>();

export const refreshToken = (): void => {
    const prevLogin = isLoggedIn.value;
    const localStorageToken = localStorage.getItem(ACCESS_TOKEN);
    const clientToken = getClient()?.get_token();
    if (localStorageToken && clientToken !== localStorageToken) {
        getClient()?.set_token(localStorageToken);
    }
    if (accessToken.value !== localStorageToken) {
        accessToken.value = localStorageToken ?? undefined;
    }

    if (!isLoggedIn.value && prevLogin) {
        window.location.reload();
    }
};

export const setLocalStorageToken = (token: string): void => {
    localStorage.setItem(ACCESS_TOKEN, token);
    refreshToken();
};

const isLoggedIn = computed(() => {
    return accessToken.value !== undefined;
});

window.addEventListener("storage", refreshToken);

const login = async (
    username: string,
    password: string,
): Promise<string | undefined> => {
    return getClient()
        ?.login(username, password)
        .then((token) => {
            setLocalStorageToken(token);
            return token;
        });
};

const logout = async (): Promise<void> => {
    if (!isLoggedIn.value) {
        return;
    }
    await getClient()?.logout();
    accessToken.value = undefined;
    removeToken();
};

const token = computed(() => {
    return accessToken.value;
});
export const useAuth = () => {
    return {
        isLoggedIn,
        login,
        logout,
        token,
    };
};
