import { client, removeToken } from "@agdb-studio/api/src/api";
import { ACCESS_TOKEN } from "@agdb-studio/api/src/constants";
import { computed, ref, watch } from "vue";
import type { LoginProps } from "@agnesoft/agdb_api/client";

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

  console.log(
    "refreshToken",
    accessToken.value,
    "prevLogin:",
    prevLogin,
    "isLoggedIn:",
    isLoggedIn.value,
  );
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

const login = async ({
  username,
  password,
  cluster,
}: LoginProps): Promise<string | undefined> => {
  return client.value
    ?.login?.({ username, password, cluster })
    .then((token) => {
      setLocalStorageToken(token);
      return token;
    });
};

const logout = async (cluster?: boolean): Promise<void> => {
  if (!isLoggedIn.value) {
    return;
  }
  await client.value?.logout(cluster).catch((error) => {
    console.error("Logout failed:", error);
  });
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
