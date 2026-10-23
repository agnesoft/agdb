<script lang="ts" setup>
import { useAdminStatus } from "@/composables/adminStatus";
import { useClusterStatus } from "@agdb-studio/cluster/src/composables/clusterStatus";
import { formatBytes, formatUptime } from "@agdb-studio/utils/src/format/format";
import SpinnerIcon from "@agdb-studio/design/src/components/icons/SpinnerIcon.vue";
import {
  MdRefresh,
  PhFillCrownSimple,
  PhFillWifiHigh,
  PhFillWifiSlash,
} from "@kalimahapps/vue-icons";
import type { ClusterStatus } from "@agnesoft/agdb_api/openapi";
import { ref } from "vue";

const {
  status: adminStatus,
  isLoading: adminLoading,
  fetchStatus: fetchAdminStatus,
} = useAdminStatus();

const {
  servers,
  isLoading: clusterLoading,
  fetchStatus: fetchClusterStatus,
} = useClusterStatus();

const refreshing = ref(false);

const refresh = async () => {
  refreshing.value = true;
  await Promise.allSettled([fetchAdminStatus(), fetchClusterStatus()]);
  refreshing.value = false;
};

const statusText = (server: ClusterStatus): string => {
  return server.status ? "Connected" : "Disconnected";
};
</script>

<template>
  <div class="admin-view">
    <div class="section-header">
      <h1>Server Status</h1>
      <button
        class="button refresh"
        title="Refresh"
        aria-label="Refresh status"
        data-testid="refresh-button"
        :disabled="refreshing"
        @click="refresh"
      >
        <MdRefresh :class="{ spinning: refreshing }" />
      </button>
    </div>

    <SpinnerIcon v-if="adminLoading" />
    <section v-else-if="adminStatus" class="server-status" data-testid="server-status">
      <div class="status-grid">
        <div class="status-card">
          <span class="label">Databases</span>
          <span class="value" data-testid="stat-dbs">{{ adminStatus.dbs }}</span>
        </div>
        <div class="status-card">
          <span class="label">Users</span>
          <span class="value" data-testid="stat-users">{{ adminStatus.users }}</span>
        </div>
        <div class="status-card">
          <span class="label">Logged In</span>
          <span class="value" data-testid="stat-logged-in">{{ adminStatus.logged_in_users }}</span>
        </div>
        <div class="status-card">
          <span class="label">Log Level</span>
          <span class="value" data-testid="stat-log-level">
            <span class="log-level-badge">{{ adminStatus.log_level }}</span>
          </span>
        </div>
        <div class="status-card">
          <span class="label">Memory</span>
          <span class="value" data-testid="stat-memory">{{ formatBytes(adminStatus.memory) }}</span>
        </div>
        <div class="status-card">
          <span class="label">Data Size</span>
          <span class="value" data-testid="stat-size">{{ formatBytes(adminStatus.size) }}</span>
        </div>
        <div class="status-card">
          <span class="label">Uptime</span>
          <span class="value" data-testid="stat-uptime">{{ formatUptime(adminStatus.uptime) }}</span>
        </div>
      </div>
    </section>
    <p v-else class="error-text" data-testid="admin-status-error">
      Failed to load server status.
    </p>

    <div class="section-header">
      <h2>Cluster Nodes</h2>
    </div>

    <SpinnerIcon v-if="clusterLoading" />
    <section v-else-if="servers.length > 0" class="cluster-status" data-testid="cluster-status">
      <table class="cluster-table">
        <thead>
          <tr>
            <th>Address</th>
            <th>Status</th>
            <th>Leader</th>
            <th>Executed Index</th>
            <th>Resync</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="server in servers"
            :key="server.address"
            :class="{ offline: !server.status }"
          >
            <td class="address-cell">{{ server.address }}</td>
            <td>
              <span class="status-indicator" :class="server.status ? 'online' : 'offline'">
                <PhFillWifiHigh v-if="server.status" class="status-icon" aria-hidden="true" />
                <PhFillWifiSlash v-else class="status-icon" aria-hidden="true" />
                {{ statusText(server) }}
              </span>
            </td>
            <td>
              <PhFillCrownSimple
                v-if="server.leader"
                class="crown-icon"
                aria-label="Leader"
                title="Leader"
              />
              <span v-else class="text-muted">—</span>
            </td>
            <td class="monospace">{{ server.executed_index }}</td>
            <td>
              <span v-if="server.resync" class="resync-active">Resyncing</span>
              <span v-else class="text-muted">—</span>
            </td>
          </tr>
        </tbody>
      </table>
    </section>
    <p v-else class="empty-text" data-testid="no-cluster">
      No cluster nodes found.
    </p>
  </div>
</template>

<style lang="less" scoped>
.admin-view {
  max-width: 900px;
  margin: 0 auto;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 1rem;

  h1,
  h2 {
    margin: 0;
  }

  h2 {
    margin-top: 1.5rem;
  }

  .refresh {
    width: 2rem;
    height: 2rem;
    font-size: 1rem;
    padding: 0;
    display: flex;
    justify-content: center;
    align-items: center;
  }
}

.status-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 0.75rem;
}

.status-card {
  background: var(--color-background-soft);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;

  .label {
    font-size: 0.8rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .value {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--color-text);
  }
}

.log-level-badge {
  display: inline-block;
  background: var(--color-background-mute);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  padding: 0.1rem 0.5rem;
  font-size: 0.9rem;
  font-weight: 500;
}

.cluster-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9rem;

  th,
  td {
    padding: 0.6rem 0.75rem;
    text-align: left;
    border-bottom: 1px solid var(--color-border);
  }

  th {
    font-weight: 600;
    color: var(--color-text-muted);
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  tr.offline {
    opacity: 0.6;
  }
}

.status-indicator {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;

  .status-icon {
    font-size: 0.9rem;
  }

  &.online {
    color: var(--success-color);

    .status-icon {
      color: var(--success-color);
    }
  }

  &.offline {
    color: var(--error-color);

    .status-icon {
      color: var(--error-color);
    }
  }
}

.crown-icon {
  color: var(--crown-color);
  font-size: 1.1rem;
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.resync-active {
  color: var(--warning-color);
  font-weight: 500;
}

.text-muted {
  color: var(--color-text-muted);
}

.monospace {
  font-family: monospace;
}

.error-text,
.empty-text {
  color: var(--color-text-muted);
  text-align: center;
  padding: 1rem 0;
}

.address-cell {
  font-family: monospace;
  font-weight: 500;
}
</style>
