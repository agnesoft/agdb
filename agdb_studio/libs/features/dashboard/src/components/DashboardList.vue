<script lang="ts" setup>
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { ChPlus, ClCloseMd, LuPencil } from "@kalimahapps/vue-icons";
import SpinnerIcon from "@agdb-studio/design/src/components/icons/SpinnerIcon.vue";
import useModal from "@agdb-studio/common/src/composables/modal/modal";
import { useContentInputs } from "@agdb-studio/common/src/composables/content/inputs";
import { KEY_MODAL } from "@agdb-studio/common/src/composables/modal/constants";
import { EMPHASIZED_CLASSNAME } from "@agdb-studio/common/src/composables/content/utils";
import { useDashboardStore } from "../composables/dashboardStore";
import { validateDashboardName } from "../utils/dashboardDocument";

const store = useDashboardStore();
const { dashboards } = store;
const { openModal } = useModal();
const { getInputValue } = useContentInputs();
const router = useRouter();

const loading = ref(true);

onMounted(async () => {
  try {
    await store.fetchDashboards();
  } catch {
    // The API error interceptor shows a notification.
  } finally {
    loading.value = false;
  }
});

const nameInput = (value = "") => ({
  input: {
    key: "name",
    label: "Name (letters, digits, '_', '-', '.')",
    type: "text" as const,
    autofocus: true,
    required: true,
    value,
    rules: [(v: string) => validateDashboardName(String(v ?? "").trim())],
  },
});

const inputName = () => String(getInputValue(KEY_MODAL, "name") ?? "").trim();

const create = () =>
  openModal({
    header: "New dashboard",
    content: [nameInput()],
    onConfirm: async () => {
      const name = inputName();
      await store.createDashboard(name);
      await router.push({ name: "dashboard", params: { name } });
      return true;
    },
  });

const rename = (name: string) =>
  openModal({
    header: `Rename dashboard ${name}`,
    content: [nameInput(name)],
    onConfirm: async () => {
      await store.renameDashboard(name, inputName());
      return true;
    },
  });

const remove = (name: string) =>
  openModal({
    header: "Delete dashboard",
    content: [
      {
        paragraph: [
          { text: "Delete dashboard " },
          { text: name, className: EMPHASIZED_CLASSNAME },
          { text: "? The databases it shows are not changed." },
        ],
      },
    ],
    onConfirm: async () => {
      await store.removeDashboard(name);
      return true;
    },
  });

const formatDate = (seconds: number) =>
  seconds ? new Date(seconds * 1000).toLocaleString() : "";
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
    <p
      v-else-if="!dashboards.length"
      class="empty"
      data-testid="dashboards-empty"
    >
      No dashboards yet. Create one, or pin a query from the query page.
    </p>
    <ul v-else class="items">
      <li v-for="dashboard of dashboards" :key="dashboard.name" class="item">
        <RouterLink
          :to="{ name: 'dashboard', params: { name: dashboard.name } }"
          class="name"
        >
          {{ dashboard.name }}
        </RouterLink>
        <span class="updated">{{ formatDate(dashboard.updated) }}</span>
        <button
          type="button"
          title="Rename"
          aria-label="Rename"
          data-testid="dashboard-rename"
          @click="rename(dashboard.name)"
        >
          <LuPencil />
        </button>
        <button
          type="button"
          title="Delete"
          aria-label="Delete"
          data-testid="dashboard-remove"
          @click="remove(dashboard.name)"
        >
          <ClCloseMd />
        </button>
      </li>
    </ul>
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
.items {
  list-style: none;
  padding: 0;
}
.item {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto auto;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid var(--color-border);
  button {
    display: flex;
    align-items: center;
    border: none;
    background: transparent;
    color: var(--color-text);
    cursor: pointer;
  }
}
.name {
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
}
.updated {
  color: var(--color-text-muted);
  font-size: 0.8rem;
}
</style>
