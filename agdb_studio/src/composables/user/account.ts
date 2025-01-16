import { client } from "@/services/api.service";
import { useAuth } from "@/composables/user/auth";
import { ref, watch } from "vue";
import type { AxiosResponse } from "axios";

const username = ref<string | undefined>(undefined);
const admin = ref<boolean>(false);

const clearStatus = () => {
    username.value = undefined;
    admin.value = false;
};

const { isLoggedIn, token } = useAuth();
const fetchUserStatus = async () => {
    if (!isLoggedIn.value) {
        clearStatus();
        return;
    }

    client.value?.user_status()?.then((status) => {
        username.value = status.data.username;
        admin.value = status.data.admin;
    });
};
watch(() => token.value, fetchUserStatus, { immediate: true });

const changePassword = async (
    oldPassword: string,
    newPassword: string,
): Promise<AxiosResponse> => {
    if (!client.value) {
        throw new Error("Client is not initialized");
    }
    return client.value.user_change_password(null, {
        password: oldPassword,
        new_password: newPassword,
    });
};

export const useAccount = (): {
    username: typeof username;
    admin: typeof admin;
    fetchUserStatus: () => Promise<void>;
    changePassword: (
        oldPassword: string,
        newPassword: string,
    ) => Promise<AxiosResponse>;
} => {
    return { username, admin, fetchUserStatus, changePassword };
};
