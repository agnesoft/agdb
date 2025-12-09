import { ref, computed, onMounted, onUnmounted, type Ref } from "vue";
import { client, checkClient } from "@agdb-studio/api/src/api";
import { createLogger } from "@agdb-studio/utils/src/logger/logger";
import type { ClusterStatus } from "@agnesoft/agdb_api/openapi";

export type OverallStatus = "red" | "amber" | "green" | "unknown";

const logger = createLogger("ClusterStatus");

const POLL_INTERVAL = 15000; // 15 seconds

const servers = ref<ClusterStatus[]>([]);
const isLoading = ref(true);
const lastUpdated = ref<Date | null>(null);

let pollInterval: ReturnType<typeof setInterval> | null = null;

export const useClusterStatus = () => {
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
    fetchStatus,
    startPolling,
    stopPolling,
  };
};
