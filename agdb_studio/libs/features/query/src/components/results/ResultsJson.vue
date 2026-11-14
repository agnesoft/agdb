<script lang="ts" setup>
import type { Components } from "@agnesoft/agdb_api";
import { computed } from "vue";
import { addNotification } from "@agdb-studio/notification/src/composables/notificationStore";
import { copyText } from "../../utils/clipboard";

const props = defineProps<{
  results: Components.Schemas.QueryResult[];
}>();

const json = computed(() => JSON.stringify(props.results, null, 2));

const copy = async () => {
  if (await copyText(json.value)) {
    addNotification({ type: "success", title: "Copied to clipboard" });
  } else {
    addNotification({ type: "error", title: "Copy to clipboard failed" });
  }
};
</script>

<template>
  <div class="results-json" data-testid="results-json">
    <button
      type="button"
      class="button copy-button"
      data-testid="json-copy"
      @click="copy"
    >
      Copy
    </button>
    <pre>{{ json }}</pre>
  </div>
</template>

<style lang="less" scoped>
.results-json {
  position: relative;
  height: 100%;
  min-height: 20rem;
  max-height: calc(100vh - 14rem);
  overflow: auto;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background-color: var(--graph-background);
  pre {
    padding: 0.75rem 1rem;
    font-size: 0.85rem;
  }
}
.copy-button {
  position: sticky;
  float: right;
  top: 0.5rem;
  margin: 0.5rem;
}
</style>
