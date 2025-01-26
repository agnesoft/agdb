import { client, removeToken } from "@/services/api.service";
import { ACCESS_TOKEN } from "@/constants";
import { computed, ref, watch } from "vue";

const accessToken = ref<string>();

const isLoggedIn = computed(() => {
    return accessToken.value !== undefined;
});

export const refreshToken = (): void => {
    const prevLogin = isLoggedIn.value;
    const localStorageToken = localStorage.getItem(ACCESS_TOKEN);
    const clientToken = client.value?.get_token();
    if (localStorageToken && clientToken !== localStorageToken) {
        client.value?.set_token(localStorageToken);
    }
    if (accessToken.value !== localStorageToken) {
        accessToken.value = localStorageToken ?? undefined;
    }

    if (!isLoggedIn.value && prevLogin) {
        window.location.reload();
    }
};
refreshToken();

watch(client, refreshToken);

export const setLocalStorageToken = (token: string): void => {
    localStorage.setItem(ACCESS_TOKEN, token);
    refreshToken();
};

window.addEventListener("storage", refreshToken);

const login = async (
    username: string,
    password: string,
): Promise<string | undefined> => {
    return client.value?.login(username, password).then((token) => {
        setLocalStorageToken(token);
        return token;
    });
};

const logout = async (): Promise<void> => {
    if (!isLoggedIn.value) {
        return;
    }
    await client.value?.logout();
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
