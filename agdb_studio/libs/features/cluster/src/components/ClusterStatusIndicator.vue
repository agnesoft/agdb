<script lang="ts" setup>
import { computed, ref } from "vue";
import { useClusterStatus } from "../composables/clusterStatus";
import {
  PhFillArrowClockwise,
  PhFillCheckCircle,
  PhFillCrownSimple,
  PhFillQuestion,
  PhFillWifiHigh,
  PhFillWifiSlash,
  PhFillXCircle,
  FaUserXmark,
  FaUserCheck,
  FaUser,
} from "@kalimahapps/vue-icons";
import FadeTransition from "@agdb-studio/design/src/components/transitions/FadeTransition.vue";
import type { ClusterStatus } from "@agnesoft/agdb_api/openapi";

const {
  servers,
  overallStatus,
  isLoading,
  fetchStatus,
  switchingServerAddress,
  activeServer,
  activeNodeLabel,
  isServerActive,
  isUserLoggedInOnServer,
  switchToServer,
} = useClusterStatus();

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

const handleServerClick = async (server: ClusterStatus) => {
  if (!server.status) {
    return;
  }
  await switchToServer(server);
};

const serverStatusText = (server: ClusterStatus): string => {
  if (switchingServerAddress.value === server.address) {
    return "Connecting...";
  }
  if (!server.status) {
    return "Offline";
  }
  if (isServerActive(server)) {
    return "Active";
  }
  return "Online";
};

const serverLoginText = (server: ClusterStatus): string => {
  const isLoggedIn = isUserLoggedInOnServer(server);
  if (isLoggedIn === null) {
    return "Unknown";
  }
  return isLoggedIn ? "Logged in" : "Logged out";
};

const serverLoginClass = (server: ClusterStatus): string => {
  const isLoggedIn = isUserLoggedInOnServer(server);
  if (isLoggedIn === null) {
    return "unknown";
  }
  return isLoggedIn ? "loggedIn" : "loggedOut";
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
    role="button"
    tabindex="0"
    aria-label="Toggle cluster status details"
    @click="handleClick"
    @keydown.enter="handleClick"
    @keydown.space.prevent="handleClick"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <FadeTransition>
      <div v-if="leaderPosition !== -1">
        <div class="connected-to">
          Connected to:
          <strong>
            {{ activeServer?.address ?? activeNodeLabel }}
            <PhFillCrownSimple
              v-if="activeServer?.leader"
              class="crown-icon"
              data-testid="active-server-crown-icon"
              aria-label="Connected leader server"
              title="Connected leader server"
            />
          </strong>
        </div>
      </div>
    </FadeTransition>
    <div
      class="status-indicator"
      :class="overallStatus"
      :title="`Cluster status: ${statusText} — connected to ${activeNodeLabel}`"
    />

    <FadeTransition>
      <div v-if="showDetails" class="status-details-wrapper">
        <div class="status-details">
          <div v-if="isLoading" class="loading">Loading...</div>
          <div v-else-if="servers.length === 0" class="no-servers">
            No clusters found
          </div>
          <div v-else class="servers-list">
            <div
              v-for="server in servers"
              :key="server.address"
              class="server-item"
              :class="{
                offline: !server.status,
                active: isServerActive(server),
                connecting: switchingServerAddress === server.address,
                disabled:
                  !server.status ||
                  switchingServerAddress === server.address ||
                  isServerActive(server) ||
                  serverLoginClass(server) === 'loggedOut',
              }"
              :title="`Server: ${server.address} \nNode status: ${serverStatusText(server)} \nLogin status: ${serverLoginText(server)}`"
              :role="server.status ? 'button' : undefined"
              :tabindex="server.status ? 0 : -1"
              @click.stop="handleServerClick(server)"
              @keydown.enter.stop.prevent="handleServerClick(server)"
            >
              <span class="server-address">{{ server.address }}</span>
              <PhFillCrownSimple
                v-if="server.leader"
                class="crown-icon"
                data-testid="crown-icon"
                aria-label="Leader server"
                title="Leader server"
              />
              <span
                class="server-status"
                :class="{
                  online:
                    server.status && switchingServerAddress !== server.address,
                  offline: !server.status,
                  connecting: switchingServerAddress === server.address,
                }"
                :title="`Node status: ${serverStatusText(server)}`"
              >
                <PhFillArrowClockwise
                  v-if="switchingServerAddress === server.address"
                  class="status-icon spinning"
                  aria-hidden="true"
                />
                <PhFillWifiSlash
                  v-else-if="!server.status"
                  class="status-icon"
                  aria-hidden="true"
                />
                <PhFillWifiHigh v-else class="status-icon" aria-hidden="true" />
                <span class="sr-only">{{ serverStatusText(server) }}</span>
              </span>
              <span
                class="server-login"
                :class="serverLoginClass(server)"
                :title="`Login status: ${serverLoginText(server)}`"
              >
                <FaUserCheck
                  v-if="serverLoginClass(server) === 'loggedIn'"
                  class="login-icon"
                  aria-hidden="true"
                />
                <FaUserXmark
                  v-else-if="serverLoginClass(server) === 'loggedOut'"
                  class="login-icon"
                  aria-hidden="true"
                />
                <PhFillQuestion v-else class="login-icon" aria-hidden="true" />
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
    background-color: var(--error-color);
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
  font-size: 0.8rem;
  color: var(--color-text-muted);
  padding-bottom: 0.5rem;
  margin-bottom: 0.5rem;
  border-bottom: 1px solid var(--color-border);

  strong {
    color: var(--color-text);
    word-break: break-all;
  }
}

.active-node-label {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--color-text);
  font-family: monospace;
}
.connected-to {
  font-size: 0.8rem;
  color: var(--color-text-muted);

  strong {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    color: var(--color-text);
    font-family: monospace;
  }
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
  transition:
    background-color 0.2s ease,
    transform 0.2s ease;

  &:not(.disabled) {
    cursor: pointer;

    &:hover {
      background: color-mix(
        in srgb,
        var(--color-background-soft) 80%,
        var(--color-text) 20%
      );
      transform: translateY(-1px);
    }
  }

  &.active {
    outline: 1px solid var(--green);
  }

  &.connecting {
    opacity: 0.7;
  }

  &.offline {
    cursor: not-allowed;

    .server-status {
      color: var(--red-2);
    }
  }
}

.server-address {
  flex: 1;
  font-weight: 500;
}

.crown-icon {
  color: #d4af37;
}

.server-status {
  display: inline-flex;
  align-items: center;

  .status-icon {
    font-size: 0.9rem;
    color: var(--green);
  }

  &.offline .status-icon {
    color: var(--red-2);
  }

  &.connecting .status-icon {
    color: var(--orange);
  }
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

.spinning {
  animation: status-spin 1s linear infinite;
}

@keyframes status-spin {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

.server-login {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  color: var(--color-text-muted);
  font-size: 0.8rem;

  .login-icon {
    font-size: 0.9rem;
    color: var(--color-border);
  }

  &.loggedIn {
    color: var(--green);

    .login-icon {
      color: var(--green);
    }
  }

  &.loggedOut {
    color: var(--red-2);

    .login-icon {
      color: var(--red-2);
    }
  }

  &.unknown {
    color: var(--color-text-muted);

    .login-icon {
      color: var(--color-border);
    }
  }
}
</style>
