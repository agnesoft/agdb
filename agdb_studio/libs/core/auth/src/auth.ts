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
type StudioLoginProps = {
  username: string;
  password: string;
  cluster?: boolean;
  server?: string;
};

const accessToken = ref<string>();

const isLoggedIn = computed(() => {
  return accessToken.value !== undefined;
});

export const refreshToken = (): void => {
  const prevLogin = isLoggedIn.value;
  const localStorageToken = localStorage.getItem(ACCESS_TOKEN);
  if (client.value && localStorageToken) {
    client.value.token = localStorageToken;
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

  const api = client.value;
  if (!api) {
    return undefined;
  }

  await (cluster
    ? api.cluster_user_login({ username, password })
    : api.user_login({ username, password }));
  const token = api.token;
  if (token) {
    resetUnauthorized();
    setLocalStorageToken(token);
  }
  return token;
};

const logoutCall = (
  cluster?: boolean,
  session?: string,
): Promise<number> | undefined => {
  const api = client.value;
  if (!api) {
    return undefined;
  }
  if (session === "all") {
    return cluster ? api.cluster_user_logout_all() : api.user_logout_all();
  }
  if (session === "others") {
    return cluster
      ? api.cluster_user_logout_others()
      : api.user_logout_others();
  }
  if (session) {
    return cluster
      ? api.cluster_user_logout_session({ session })
      : api.user_logout_session({ session });
  }
  return cluster ? api.cluster_user_logout() : api.user_logout();
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

  await logoutCall(cluster, session)?.catch((error) => {
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
    await logoutCall(cluster)?.catch((error) => {
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
