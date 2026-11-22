<script lang="ts" setup>
import { ref } from "vue";
import type { DashboardSummary } from "@agnesoft/agdb_api";
import {
  BsGlobe,
  BsStar,
  BsStarFill,
  ClCloseMd,
  LuLogOut,
  LuPencil,
  LuUsers,
} from "@kalimahapps/vue-icons";
import useModal from "@agdb-studio/common/src/composables/modal/modal";
import { EMPHASIZED_CLASSNAME } from "@agdb-studio/common/src/composables/content/utils";
import {
  adminMode,
  dashboardLabel,
  dashboardRoute,
  isOwner,
  sameDashboard,
  useDashboardStore,
  type DashboardKey,
} from "../composables/dashboardStore";
import { dashboardNameInput, modalInputValue } from "../utils/dashboardInputs";
import DashboardUsers from "./DashboardUsers.vue";

/** One section of the dashboard overview, e.g. "Shared with me". */
defineProps<{ title: string; dashboards: DashboardSummary[] }>();

const store = useDashboardStore();
const { home } = store;
const { openModal } = useModal();

/** The dashboard whose users are shown. */
const showUsers = ref<DashboardKey>();
/** Home is personal: the admin view has none. */
const admin = adminMode();

const ROLE_LABELS = { admin: "Admin", write: "Read/Write", read: "Read only" };

const keyOf = (dashboard: DashboardSummary): DashboardKey => ({
  owner: dashboard.owner,
  name: dashboard.name,
});

const rename = (key: DashboardKey) =>
  openModal({
    header: `Rename dashboard ${dashboardLabel(key)}`,
    content: [dashboardNameInput(key.name)],
    onConfirm: async () => {
      await store.renameDashboard(key, modalInputValue("name"));
      return true;
    },
  });

const remove = (key: DashboardKey) =>
  openModal({
    header: "Delete dashboard",
    content: [
      {
        paragraph: [
          { text: "Delete dashboard " },
          { text: dashboardLabel(key), className: EMPHASIZED_CLASSNAME },
          {
            text: "? Users it is shared with lose it too. The databases it shows are not changed.",
          },
        ],
      },
    ],
    onConfirm: async () => {
      await store.removeDashboard(key);
      return true;
    },
  });

const leave = (key: DashboardKey) =>
  openModal({
    header: "Leave dashboard",
    content: [
      {
        paragraph: [
          { text: "Stop using dashboard " },
          { text: dashboardLabel(key), className: EMPHASIZED_CLASSNAME },
          { text: "? Its owner or an admin can share it again." },
        ],
      },
    ],
    onConfirm: async () => {
      await store.leaveDashboard(key);
      return true;
    },
  });

const togglePublic = (dashboard: DashboardSummary) => {
  const key = keyOf(dashboard);
  if (dashboard.public) {
    openModal({
      header: "Make dashboard private",
      content: [
        {
          paragraph: [
            { text: "Make dashboard " },
            { text: dashboardLabel(key), className: EMPHASIZED_CLASSNAME },
            {
              text: " private? Only the users it is shared with can view it then.",
            },
          ],
        },
      ],
      onConfirm: async () => {
        await store.setPublic(key, false);
        return true;
      },
    });
  } else {
    openModal({
      header: "Make dashboard public",
      content: [
        {
          paragraph: [
            { text: "Make dashboard " },
            { text: dashboardLabel(key), className: EMPHASIZED_CLASSNAME },
            {
              text: " public? All users can view it (read only). Its widgets run with each viewer's own database access.",
            },
          ],
        },
      ],
      onConfirm: async () => {
        await store.setPublic(key, true);
        return true;
      },
    });
  }
};

const isHome = (key: DashboardKey) => sameDashboard(home.value, key);

const toggleHome = async (key: DashboardKey) => {
  try {
    await store.setHome(isHome(key) ? undefined : key);
  } catch {
    // The API error interceptor shows a notification.
  }
};

const toggleUsers = (key: DashboardKey) => {
  showUsers.value = sameDashboard(showUsers.value, key) ? undefined : key;
};

const formatDate = (seconds: number) =>
  seconds ? new Date(seconds * 1000).toLocaleString() : "";
</script>

<template>
  <section class="dashboard-table" data-testid="dashboard-table">
    <h3>{{ title }}</h3>
    <slot v-if="!dashboards.length" name="empty" />
    <ul v-else class="items">
      <li
        v-for="dashboard of dashboards"
        :key="`${dashboard.owner}/${dashboard.name}`"
        class="entry"
        data-testid="dashboard-entry"
      >
        <div class="item">
          <span v-if="admin" />
          <button
            v-else
            type="button"
            class="home"
            :class="{ active: isHome(keyOf(dashboard)) }"
            :title="
              isHome(keyOf(dashboard))
                ? 'Opens at start (click to stop)'
                : 'Open at start'
            "
            :aria-label="
              isHome(keyOf(dashboard))
                ? 'Unset home dashboard'
                : 'Set as home dashboard'
            "
            :aria-pressed="isHome(keyOf(dashboard))"
            data-testid="dashboard-home"
            @click="toggleHome(keyOf(dashboard))"
          >
            <BsStarFill v-if="isHome(keyOf(dashboard))" />
            <BsStar v-else />
          </button>
          <RouterLink :to="dashboardRoute(dashboard)" class="name">
            {{ dashboardLabel(dashboard) }}
          </RouterLink>
          <span class="badges">
            <span
              v-if="!isOwner(dashboard) && dashboard.member"
              class="badge"
              data-testid="dashboard-role"
            >
              {{ ROLE_LABELS[dashboard.role] }}
            </span>
            <span
              v-if="!isOwner(dashboard) && dashboard.public"
              class="badge"
              title="All users can view this dashboard"
              data-testid="dashboard-public-badge"
            >
              Public
            </span>
          </span>
          <span class="updated">{{ formatDate(dashboard.updated) }}</span>
          <button
            v-if="isOwner(dashboard)"
            type="button"
            class="public"
            :class="{ active: dashboard.public }"
            :title="
              dashboard.public
                ? 'Public: all users can view it (click to make private)'
                : 'Private: only its users can view it (click to make public)'
            "
            :aria-label="dashboard.public ? 'Make private' : 'Make public'"
            :aria-pressed="dashboard.public"
            data-testid="dashboard-public"
            @click="togglePublic(dashboard)"
          >
            <BsGlobe />
          </button>
          <span v-else />
          <button
            v-if="dashboard.member"
            type="button"
            title="Users"
            aria-label="Users"
            data-testid="dashboard-users-toggle"
            :aria-expanded="sameDashboard(showUsers, keyOf(dashboard))"
            @click="toggleUsers(keyOf(dashboard))"
          >
            <LuUsers />
          </button>
          <span v-else />
          <template v-if="isOwner(dashboard)">
            <button
              type="button"
              title="Rename"
              aria-label="Rename"
              data-testid="dashboard-rename"
              @click="rename(keyOf(dashboard))"
            >
              <LuPencil />
            </button>
            <button
              type="button"
              title="Delete"
              aria-label="Delete"
              data-testid="dashboard-remove"
              @click="remove(keyOf(dashboard))"
            >
              <ClCloseMd />
            </button>
          </template>
          <template v-else>
            <span />
            <button
              v-if="dashboard.member"
              type="button"
              title="Leave"
              aria-label="Leave"
              data-testid="dashboard-leave"
              @click="leave(keyOf(dashboard))"
            >
              <LuLogOut />
            </button>
            <span v-else />
          </template>
        </div>
        <DashboardUsers
          v-if="sameDashboard(showUsers, keyOf(dashboard))"
          :owner="dashboard.owner"
          :name="dashboard.name"
          :role="dashboard.role"
        />
      </li>
    </ul>
  </section>
</template>

<style lang="less" scoped>
h3 {
  margin: 1rem 0 0.25rem;
  font-size: 1rem;
}
.items {
  list-style: none;
  padding: 0;
  margin: 0;
}
.entry {
  border-bottom: 1px solid var(--color-border);
}
.item {
  display: grid;
  grid-template-columns:
    1.25rem minmax(0, 1fr)
    auto auto 1.25rem 1.25rem 1.25rem 1.25rem;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  button {
    display: flex;
    align-items: center;
    border: none;
    background: transparent;
    color: var(--color-text);
    cursor: pointer;
    padding: 0;
  }
}
.name {
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
}
.home,
.public {
  color: var(--color-text-muted);
  &.active {
    color: var(--link-color);
  }
}
.badges {
  display: flex;
  gap: 0.25rem;
}
.badge {
  color: var(--color-text-muted);
  font-size: 0.8rem;
  border: 1px solid var(--color-border);
  border-radius: 0.25rem;
  padding: 0 0.35rem;
}
.updated {
  color: var(--color-text-muted);
  font-size: 0.8rem;
}
</style>
