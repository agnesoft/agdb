import { ref, computed, onMounted, onUnmounted, type Ref } from "vue";
import { AgdbApi } from "@agnesoft/agdb_api";
import {
  apiUrl,
  client,
  checkClient,
  reconnectClient,
} from "@agdb-studio/api/src/api";
import { ACCESS_TOKEN } from "@agdb-studio/api/src/constants";
import { resolveServerUrl } from "@agdb-studio/api/src/serverUrl";
import { createLogger } from "@agdb-studio/utils/src/logger/logger";
import type { ClusterStatus } from "@agnesoft/agdb_api/openapi";

export type OverallStatus = "red" | "amber" | "green" | "unknown";

const logger = createLogger("ClusterStatus");

const POLL_INTERVAL = 15000; // 15 seconds

const servers = ref<ClusterStatus[]>([]);
const isLoading = ref(true);
const lastUpdated = ref<Date | null>(null);
const switchingServerAddress = ref<string | null>(null);
const loggedInByServerAddress = ref<Record<string, boolean | null>>({});

let pollInterval: ReturnType<typeof setInterval> | null = null;

export const useClusterStatus = () => {
  // When the cluster reports internal hostnames (e.g. agdb0:3000) but the UI
  // connects via localhost, only comparing ports is reliable for active-node
  // detection. We therefore normalize to just the port.
  const normalizeAddress = (address: string): string => {
    try {
      const parsed = new URL(
        address.includes("://") ? address : `http://${address}`,
      );
      return parsed.port || "80";
    } catch {
      return address.toLowerCase();
    }
  };

  const overallStatus = computed((): OverallStatus => {
    if (isLoading.value) {
      return "unknown";
    }

    if (servers.value.length === 0) {
      return "red";
    }

    const allDown = servers.value.every((server) => !server.status);
    if (allDown) {
      return "red";
    }

    const hasLeader = servers.value.some((server) => server.leader);
    const anyDown = servers.value.some((server) => !server.status);

    if (!hasLeader || anyDown) {
      return "amber";
    }

    return "green";
  });

  const fetchStatus = async (): Promise<void> => {
    try {
      checkClient(client);
      const response = await client.value.cluster_status();
      servers.value = response.data;

      const token =
        client.value.get_token() ?? localStorage.getItem(ACCESS_TOKEN) ?? "";

      const loginStatusEntries = await Promise.all(
        response.data.map(async (server) => {
          if (!server.status) {
            return [server.address, null] as const;
          }

          try {
            const serverClient = await AgdbApi.client(
              resolveServerUrl(apiUrl.value, server.address),
            );
            if (token) {
              serverClient.set_token(token);
            }
            const status = await serverClient.user_status();
            return [server.address, Boolean(status.data.login)] as const;
          } catch {
            return [server.address, false] as const;
          }
        }),
      );

      loggedInByServerAddress.value = Object.fromEntries(loginStatusEntries);
      lastUpdated.value = new Date();
      logger.debug("Cluster status fetched:", servers.value.length, "servers");
    } catch (error) {
      logger.error(
        "Failed to fetch cluster status:",
        error instanceof Error ? error.message : String(error),
      );
      servers.value = [];
      loggedInByServerAddress.value = {};
    } finally {
      isLoading.value = false;
    }
  };

  const activeAddress = computed(() => {
    return normalizeAddress(apiUrl.value);
  });

  const isServerActive = (server: ClusterStatus): boolean => {
    return normalizeAddress(server.address) === activeAddress.value;
  };

  const activeServer = computed((): ClusterStatus | undefined => {
    return servers.value.find(isServerActive);
  });

  const isUserLoggedInOnServer = (server: ClusterStatus): boolean | null => {
    return loggedInByServerAddress.value[server.address] ?? null;
  };

  const activeNodeLabel = computed((): string => {
    try {
      const url = new URL(
        apiUrl.value.includes("://") ? apiUrl.value : `http://${apiUrl.value}`,
      );
      return `:${url.port || "80"}`;
    } catch {
      return apiUrl.value;
    }
  });

  const switchToServer = async (server: ClusterStatus): Promise<void> => {
    if (!server.status || isServerActive(server)) {
      return;
    }

    switchingServerAddress.value = server.address;
    try {
      const resolvedAddress = resolveServerUrl(apiUrl.value, server.address);
      await reconnectClient(resolvedAddress);
      await fetchStatus();
      logger.info("Switched active cluster node:", server.address);
    } catch (error) {
      logger.error(
        "Failed to switch cluster node:",
        error instanceof Error ? error.message : String(error),
      );
    } finally {
      switchingServerAddress.value = null;
    }
  };

  const startPolling = (): void => {
    if (pollInterval) {
      return;
    }
    fetchStatus();
    pollInterval = setInterval(fetchStatus, POLL_INTERVAL);
    logger.debug("Started polling cluster status");
  };

  const stopPolling = (): void => {
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
      logger.debug("Stopped polling cluster status");
    }
  };

  onMounted(() => {
    startPolling();
  });

  onUnmounted(() => {
    stopPolling();
  });

  return {
    servers: servers as Ref<ClusterStatus[]>,
    overallStatus,
    isLoading: isLoading as Ref<boolean>,
    lastUpdated: lastUpdated as Ref<Date | null>,
    switchingServerAddress: switchingServerAddress as Ref<string | null>,
    loggedInByServerAddress: loggedInByServerAddress as Ref<
      Record<string, boolean | null>
    >,
    activeServer,
    activeNodeLabel,
    isServerActive,
    isUserLoggedInOnServer,
    switchToServer,
    fetchStatus,
    startPolling,
    stopPolling,
  };
};
