import { getClient } from "@/services/api.service";
import { useAuth } from "@/composables/user/auth";
import { ref, watch } from "vue";

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

    getClient()
        ?.user_status()
        ?.then((status) => {
            username.value = status.data.name;
            admin.value = status.data.admin;
        });
};
watch(() => token.value, fetchUserStatus, { immediate: true });

export const useAccount = () => {
    return { username, admin, fetchUserStatus };
};
