import { ref, onMounted, onUnmounted, type Ref } from "vue";
import { client, checkClient } from "@agdb-studio/api/src/api";
import { useAuth } from "@agdb-studio/auth/src/auth";
import { createLogger } from "@agdb-studio/utils/src/logger/logger";
import type { AdminStatus } from "@agnesoft/agdb_api/openapi";

const logger = createLogger("AdminStatus");

const POLL_INTERVAL = 15000;

const status = ref<AdminStatus | null>(null);
const isLoading = ref(true);
const lastUpdated = ref<Date | null>(null);

let pollInterval: ReturnType<typeof setInterval> | null = null;

export const useAdminStatus = () => {
  const { isLoggedIn } = useAuth();

  const fetchStatus = async (): Promise<void> => {
    if (!isLoggedIn.value) return;
    try {
      checkClient(client);
      const response = await client.value.admin_status();
      status.value = response.data;
      lastUpdated.value = new Date();
      logger.debug("Admin status fetched");
    } catch (error) {
      logger.error(
        "Failed to fetch admin status:",
        error instanceof Error ? error.message : String(error),
      );
      status.value = null;
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
    logger.debug("Started polling admin status");
  };

  const stopPolling = (): void => {
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
      logger.debug("Stopped polling admin status");
    }
  };

  onMounted(() => {
    startPolling();
  });

  onUnmounted(() => {
    stopPolling();
  });

  return {
    status: status as Ref<AdminStatus | null>,
    isLoading: isLoading as Ref<boolean>,
    lastUpdated: lastUpdated as Ref<Date | null>,
    fetchStatus,
    startPolling,
    stopPolling,
  };
};

export const resetAdminStatusState = (): void => {
  status.value = null;
  isLoading.value = true;
  lastUpdated.value = null;
};
