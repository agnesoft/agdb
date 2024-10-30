import {
    isLoggedIn,
    login as loginService,
    logout as logoutService,
} from "@/services/auth.service";
import { defineStore } from "pinia";
import { computed } from "vue";

export const useAccountStore = defineStore("account", () => {
    const loggedIn = computed(() => {
        return isLoggedIn();
    });
    const login = async (
        username: string,
        password: string,
    ): Promise<string | undefined> => {
        return loginService(username, password);
    };
    const logout = async () => {
        return logoutService();
    };

    return { loggedIn, login, logout };
});
