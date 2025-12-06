<script lang="ts" setup>
import { ref } from "vue";
import { useClusterStatus } from "../composables/clusterStatus";
import CrownIcon from "@agdb-studio/design/src/components/icons/CrownIcon.vue";

const { servers, overallStatus, isLoading, fetchStatus } = useClusterStatus();

const showDetails = ref(false);

const statusColor = (status: typeof overallStatus.value): string => {
  switch (status) {
    case "green":
      return "var(--green-1)";
    case "amber":
      return "var(--orange-1)";
    case "red":
      return "var(--red-1)";
    default:
      return "var(--color-border)";
  }
};

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
</script>

<template>
  <div
    class="cluster-status"
    @click="handleClick"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <div
      class="status-indicator"
      :style="{ backgroundColor: statusColor(overallStatus) }"
      :title="`Cluster status: ${overallStatus}`"
    />

    <div v-if="showDetails" class="status-details">
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
          <CrownIcon v-if="server.leader" class="crown-icon" />
          <span class="server-status">
            {{ server.status ? "Online" : "Offline" }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="less" scoped>
.cluster-status {
  position: relative;
  display: inline-block;
  cursor: pointer;
}

.status-indicator {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 2px solid var(--color-background);
  transition: all 0.3s ease;

  &:hover {
    transform: scale(1.2);
  }
}

.status-details {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 12px;
  min-width: 250px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 1000;
}

.loading,
.no-servers {
  color: var(--color-text-muted);
  text-align: center;
  padding: 8px 0;
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
    opacity: 0.6;

    .server-status {
      color: var(--red-1);
    }
  }
}

.server-address {
  flex: 1;
  font-weight: 500;
}

.crown-icon {
  width: 16px;
  height: 16px;
  color: var(--yellow-1);
  flex-shrink: 0;
}

.server-status {
  color: var(--green-1);
  font-size: 0.85rem;
}
</style>
