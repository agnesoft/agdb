<script lang="ts" setup>
import { computed, ref } from "vue";
import { useClusterStatus } from "../composables/clusterStatus";
import { PhFillCrownSimple } from "@kalimahapps/vue-icons";
import FadeTransition from "@agdb-studio/design/src/components/transitions/FadeTransition.vue";

const { servers, overallStatus, isLoading, fetchStatus } = useClusterStatus();

const showDetails = ref(false);

const handleClick = async () => {
  showDetails.value = !showDetails.value;
  if (showDetails.value) {
    await fetchStatus();
  }
};

const handleMouseEnter = () => {
  showDetails.value = true;
  fetchStatus();
};

const handleMouseLeave = () => {
  showDetails.value = false;
};

const leaderPosition = computed(() => {
  return servers.value.findIndex((server) => server.leader);
});

const statusText = computed(() => {
  switch (overallStatus.value) {
    case "green":
      return "Healthy";
    case "amber":
      return "Degraded";
    case "red":
      return "Unhealthy";
    default:
      return "Unknown";
  }
});
</script>

<template>
  <div
    class="cluster-status"
    @click="handleClick"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <FadeTransition>
      <span v-if="leaderPosition !== -1"> Cluster [{{ leaderPosition }}] </span>
    </FadeTransition>
    <div
      class="status-indicator"
      :class="overallStatus"
      :title="`Cluster status: ${statusText}`"
    />

    <FadeTransition>
      <div v-if="showDetails" class="status-details-wrapper">
        <div class="status-details">
          <div v-if="isLoading" class="loading">Loading...</div>
          <div v-else-if="servers.length === 0" class="no-servers">
            No servers found
          </div>
          <div v-else class="servers-list">
            <div
              v-for="server in servers"
              :key="server.address"
              class="server-item"
              :class="{ offline: !server.status }"
            >
              <span class="server-address">{{ server.address }}</span>
              <PhFillCrownSimple
                v-if="server.leader"
                data-testid="crown-icon"
              />
              <span class="server-status">
                {{ server.status ? "Online" : "Offline" }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </FadeTransition>
  </div>
</template>

<style lang="less" scoped>
.cluster-status {
  position: relative;
  display: inline-flex;
  cursor: pointer;
  align-items: center;
  gap: 0.6rem;
}

.status-indicator {
  width: 1rem;
  height: 1rem;
  border-radius: 50%;
  border: 2px solid var(--color-background);
  transition: all 0.3s ease;

  &:hover {
    transform: scale(1.2);
  }

  &.green {
    background-color: var(--green);
  }
  &.amber {
    background-color: var(--orange);
  }
  &.red {
    background-color: var(--red);
  }
  &.unknown {
    background-color: var(--color-border);
  }
}

.status-details-wrapper {
  position: absolute;
  top: calc(100% - 0.5rem);
  right: 0;
  z-index: 1000;
}

.status-details {
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 0.75rem;
  min-width: 250px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  margin: 1rem 0 0 0;
}

.loading,
.no-servers {
  color: var(--color-text-muted);
  text-align: center;
  padding: 0.5rem 0;
}

.servers-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.server-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  background: var(--color-background-soft);
  border-radius: 4px;
  font-size: 0.9rem;

  &.offline {
    .server-status {
      color: var(--red-2);
    }
  }
}

.server-address {
  flex: 1;
  font-weight: 500;
}

.server-status {
  color: var(--green-1);
  font-size: 0.85rem;
}
</style>
