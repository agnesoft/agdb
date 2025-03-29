import { client } from "@/services/api.service";
import type { UserStatus } from "@agnesoft/agdb_api/openapi";
import { ref } from "vue";
import { addNotification } from "../../../libs/notification/src/composables/notificationStore";

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
