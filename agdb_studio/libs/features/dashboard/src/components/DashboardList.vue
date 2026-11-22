<script lang="ts" setup>
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { ChPlus } from "@kalimahapps/vue-icons";
import SpinnerIcon from "@agdb-studio/design/src/components/icons/SpinnerIcon.vue";
import useModal from "@agdb-studio/common/src/composables/modal/modal";
import {
  adminMode,
  dashboardRoute,
  useDashboardStore,
} from "../composables/dashboardStore";
import { dashboardNameInput, modalInputValue } from "../utils/dashboardInputs";
import DashboardTable from "./DashboardTable.vue";

const store = useDashboardStore();
const { dashboards, mine, shared, publicOnly } = store;
const { openModal } = useModal();
const router = useRouter();

/** The server admin manages every user's dashboards in one table. */
const admin = adminMode();
const loading = ref(true);

onMounted(async () => {
  try {
    await Promise.all(
      admin
        ? [store.fetchDashboards()]
        : [store.fetchDashboards(), store.fetchHome()],
    );
  } catch {
    // The API error interceptor shows a notification.
  } finally {
    loading.value = false;
  }
});

const ownerInput = {
  input: {
    key: "owner",
    label: "Owner (username)",
    type: "text" as const,
    autofocus: true,
    required: true,
  },
};

const create = () =>
  openModal({
    header: "New dashboard",
    content: admin
      ? [ownerInput, dashboardNameInput("", false)]
      : [dashboardNameInput()],
    onConfirm: async () => {
      const owner = admin ? modalInputValue("owner") : undefined;
      // The admin creates for a user: without one, nothing is created.
      if (owner === "") return false;
      const key = await store.createDashboard(modalInputValue("name"), owner);
      await router.push(dashboardRoute(key));
      return true;
    },
  });
</script>

<template>
  <div class="dashboard-list">
    <header class="toolbar">
      <h2>Dashboards</h2>
      <button
        type="button"
        class="button button-success"
        data-testid="dashboard-create"
        @click="create"
      >
        <ChPlus /> New dashboard
      </button>
    </header>
    <SpinnerIcon v-if="loading" />
    <DashboardTable
      v-else-if="admin"
      title="All dashboards"
      :dashboards="dashboards"
      data-testid="dashboards-all"
    >
      <template #empty>
        <p class="empty" data-testid="dashboards-empty">No dashboards yet.</p>
      </template>
    </DashboardTable>
    <template v-else>
      <DashboardTable
        title="My dashboards"
        :dashboards="mine"
        data-testid="dashboards-mine"
      >
        <template #empty>
          <p class="empty" data-testid="dashboards-empty">
            No dashboards yet. Create one, or pin a query from the query page.
          </p>
        </template>
      </DashboardTable>
      <DashboardTable
        v-if="shared.length"
        title="Shared with me"
        :dashboards="shared"
        data-testid="dashboards-shared"
      />
      <DashboardTable
        v-if="publicOnly.length"
        title="Public"
        :dashboards="publicOnly"
        data-testid="dashboards-public"
      />
    </template>
  </div>
</template>

<style lang="less" scoped>
.dashboard-list {
  max-width: 800px;
  width: 100%;
  margin: 0 auto;
}
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  .button {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
  }
}
.empty {
  color: var(--color-text-muted);
  text-align: center;
}
</style>
