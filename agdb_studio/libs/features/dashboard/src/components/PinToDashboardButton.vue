<script lang="ts" setup>
import { computed } from "vue";
import { ChPlus } from "@kalimahapps/vue-icons";
import useModal from "@agdb-studio/common/src/composables/modal/modal";
import { useContentInputs } from "@agdb-studio/common/src/composables/content/inputs";
import { KEY_MODAL } from "@agdb-studio/common/src/composables/modal/constants";
import { addNotification } from "@agdb-studio/notification/src/composables/notificationStore";
import { useDashboardStore } from "../composables/dashboardStore";
import { useQueryStore } from "@agdb-studio/query/src/composables/queryStore";
import {
  NewerDashboardError,
  createWidget,
  validateDashboardName,
} from "../utils/dashboardDocument";
import { hasUnfinishedRow, widgetFromQuery } from "../utils/widgetRows";

const props = defineProps<{
  owner: string;
  db: string;
  /** The query page's query store entry; its exec and context rows are pinned. */
  queryId?: string;
}>();

/** Not a valid dashboard name (a leading "." is rejected), so no dashboard can clash with it. */
const NEW_DASHBOARD = ".";

const store = useDashboardStore();
const { openModal } = useModal();
const { getInputValue } = useContentInputs();

const queryStore = useQueryStore();
const query = computed(() =>
  props.queryId ? queryStore.getQuery(props.queryId)?.value : undefined,
);
const hasQuery = computed(
  () => !!query.value?.rows.exec.some((row) => row.steps.length > 0),
);
const unfinished = computed(
  () => !!query.value && hasUnfinishedRow(query.value),
);
const pinnable = computed(() => hasQuery.value && !unfinished.value);

const text = (key: string) =>
  String(getInputValue(KEY_MODAL, key) ?? "").trim();

const pin = async () => {
  try {
    await store.fetchDashboards();
  } catch {
    return;
  }
  const names = store.dashboards.value.map((d) => d.name);
  let created: string | undefined;
  openModal({
    header: "Add to dashboard",
    content: [
      {
        input: {
          key: "dashboard",
          label: "Dashboard",
          type: "select",
          options: [
            ...names.map((n) => ({ value: n, label: n })),
            { value: NEW_DASHBOARD, label: "+ New dashboard" },
          ],
          value: names[0] ?? NEW_DASHBOARD,
        },
      },
      {
        input: {
          key: "new_name",
          label: "New dashboard name (when creating one)",
          type: "text",
          rules: [
            (value: string) =>
              text("dashboard") === NEW_DASHBOARD
                ? validateDashboardName(String(value ?? "").trim())
                : undefined,
          ],
        },
      },
      {
        input: {
          key: "title",
          label: "Widget title",
          type: "text",
          required: true,
          autofocus: true,
          value: `${props.owner}/${props.db}`,
        },
      },
    ],
    onConfirm: async () => {
      let name = text("dashboard");
      if (name === NEW_DASHBOARD) {
        name = text("new_name");
        // A retry after a failed pin must not create the dashboard again.
        if (name !== created) await store.createDashboard(name);
        created = name;
      }
      /* v8 ignore next -- @preserve: the button is disabled without a query */
      if (!query.value) return false;
      // The whole query: its rows, colours and the view shown now.
      const widget = widgetFromQuery(
        createWidget({
          owner: props.owner,
          db: props.db,
          title: text("title"),
        }),
        query.value,
      );
      try {
        await store.appendWidgets(name, [widget]);
      } catch (error) {
        // Request errors get a notification from the API error interceptor.
        if (!(error instanceof NewerDashboardError)) throw error;
        addNotification({
          type: "error",
          title: "Not added",
          message: error.message,
        });
        return false;
      }
      addNotification({
        type: "success",
        title: "Added to dashboard",
        message: `"${widget.title}" added to ${name}.`,
      });
      return true;
    },
  });
};
</script>

<template>
  <button
    type="button"
    class="button pin-to-dashboard"
    data-testid="pin-to-dashboard"
    :disabled="!pinnable"
    :title="
      pinnable
        ? 'Add this query (exec and context rows, colours, view) to a dashboard'
        : hasQuery
          ? 'Finish or remove the rows that cannot run yet to add them to a dashboard'
          : 'Write a query in the exec tab to add it to a dashboard'
    "
    @click="pin"
  >
    <ChPlus /> Add to dashboard
  </button>
</template>

<style lang="less" scoped>
.pin-to-dashboard {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
}
</style>
