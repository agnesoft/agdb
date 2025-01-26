import { client } from "@/services/api.service";
import type { UserStatus } from "agdb_api/dist/openapi";
import { ref } from "vue";
import { addNotification } from "../notification/notificationStore";

const users = ref<UserStatus[]>([]);

const fetchUsers = async () => {
    client.value?.admin_user_list().then((response) => {
        users.value = response.data;
    });
};

type AddUserParams = {
    username: string;
    password: string;
};
const addUser = async ({ username, password }: AddUserParams) => {
    client.value?.admin_user_add({ username }, { password }).then(() => {
        addNotification({
            type: "success",
            title: "User added",
            message: `User ${username} added successfully.`,
        });
    });
};

export const useUserStore = () => {
    return {
        users,
        fetchUsers,
        addUser,
    };
};
