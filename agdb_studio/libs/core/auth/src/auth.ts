import {
  apiUrl,
  client,
  notifyUnauthorized,
  reconnectClient,
  removeToken,
  resetUnauthorized,
} from "@agdb-studio/api/src/api";
import { ACCESS_TOKEN } from "@agdb-studio/api/src/constants";
import { resolveServerUrl } from "@agdb-studio/api/src/serverUrl";
import { computed, ref, watch } from "vue";
import type { LoginProps } from "@agnesoft/agdb_api/client";

type StudioLoginProps = LoginProps & {
  server?: string;
};

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
    notifyUnauthorized();
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
  server,
}: StudioLoginProps): Promise<string | undefined> => {
  if (server) {
    const resolvedServerUrl = resolveServerUrl(apiUrl.value, server);
    if (resolvedServerUrl !== apiUrl.value) {
      await reconnectClient(resolvedServerUrl);
    }
  }

  return client.value
    ?.login?.({ username, password, cluster })
    .then((token) => {
      resetUnauthorized();
      setLocalStorageToken(token);
      return token;
    });
};

const logoutSession = async (
  cluster?: boolean,
  session?: string,
): Promise<void> => {
  if (!isLoggedIn.value) {
    return;
  }

  const endsCurrentSession = !session || session === "all";

  if (endsCurrentSession) {
    accessToken.value = undefined;
    notifyUnauthorized();
  }

  const params = session ? { session } : undefined;
  const call = cluster
    ? client.value?.cluster_user_logout(params)
    : client.value?.user_logout(params);

  await call?.catch((error) => {
    console.error("Logout failed:", error);
  });

  if (endsCurrentSession) {
    removeToken();
  }
};

const logout = async (
  cluster?: boolean,
  serverLogout = true,
): Promise<void> => {
  if (!isLoggedIn.value) {
    return;
  }
  accessToken.value = undefined;
  if (serverLogout) {
    notifyUnauthorized();
    await client.value?.logout(cluster).catch((error) => {
      console.error("Logout failed:", error);
    });
  }
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
    logoutSession,
    token,
  };
};
