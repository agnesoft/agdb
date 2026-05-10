import { ref, computed, onMounted, onUnmounted, type Ref } from "vue";
import {
  apiUrl,
  client,
  checkClient,
  reconnectClient,
} from "@agdb-studio/api/src/api";
import { createLogger } from "@agdb-studio/utils/src/logger/logger";
import type { ClusterStatus } from "@agnesoft/agdb_api/openapi";

export type OverallStatus = "red" | "amber" | "green" | "unknown";

const logger = createLogger("ClusterStatus");

const POLL_INTERVAL = 15000; // 15 seconds

const servers = ref<ClusterStatus[]>([]);
const isLoading = ref(true);
const lastUpdated = ref<Date | null>(null);
const switchingServerAddress = ref<string | null>(null);

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
      lastUpdated.value = new Date();
      logger.debug("Cluster status fetched:", servers.value.length, "servers");
    } catch (error) {
      logger.error(
        "Failed to fetch cluster status:",
        error instanceof Error ? error.message : String(error),
      );
      servers.value = [];
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

  const resolveServerUrl = (server: ClusterStatus): string => {
    try {
      const current = new URL(apiUrl.value);
      const target = new URL(
        server.address.includes("://")
          ? server.address
          : `${current.protocol}//${server.address}`,
      );
      // Keep the host from the current connection (e.g. localhost)
      // but take the port from the cluster member's address.
      current.port = target.port;
      return current.toString().replace(/\/$/, "");
    } catch {
      return server.address;
    }
  };

  const switchToServer = async (server: ClusterStatus): Promise<void> => {
    if (!server.status || isServerActive(server)) {
      return;
    }

    switchingServerAddress.value = server.address;
    try {
      const resolvedAddress = resolveServerUrl(server);
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
    activeServer,
    activeNodeLabel,
    isServerActive,
    switchToServer,
    fetchStatus,
    startPolling,
    stopPolling,
  };
};
