import { client } from "@agdb-studio/api/src/api";
import { useAuth } from "./auth";
import { ref, watch } from "vue";

const TOKEN_CHECK_INTERVAL = 60000;

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

  return client.value?.user_status()?.then((status) => {
    username.value = status.data.username;
    admin.value = status.data.admin;
  });
};
watch(
  () => token.value,
  () => {
    fetchUserStatus().catch(() => {});
  },
);

let tokenCheckInterval: ReturnType<typeof setInterval> | null = null;

watch(
  isLoggedIn,
  (loggedIn) => {
    if (loggedIn) {
      if (!tokenCheckInterval) {
        tokenCheckInterval = setInterval(() => {
          fetchUserStatus().catch(() => {});
        }, TOKEN_CHECK_INTERVAL);
      }
    } else {
      if (tokenCheckInterval) {
        clearInterval(tokenCheckInterval);
        tokenCheckInterval = null;
      }
    }
  },
  { immediate: true },
);

export const useAccount = (): {
  username: typeof username;
  admin: typeof admin;
  fetchUserStatus: () => Promise<void>;
} => {
  return { username, admin, fetchUserStatus };
};
