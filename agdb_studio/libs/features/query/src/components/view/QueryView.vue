<script lang="ts" setup>
import { onMounted, provide, ref } from "vue";
import QueryBuilderTabs from "../builder/QueryBuilderTabs.vue";
import QueryResults from "../results/QueryResults.vue";
import { useQueryStore } from "../../composables/queryStore";

const queryId = ref<string>();
const queryStore = useQueryStore();
onMounted(() => {
  const baseId = crypto.randomUUID();
  queryId.value = `query-${baseId}`;
  queryStore.addQuery({ id: queryId.value });
});

provide("queryId", queryId);
</script>

<template>
  <div class="query-workspace">
    <QueryBuilderTabs />
    <QueryResults />
  </div>
</template>

<style lang="less" scoped>
// Not ".query-view": the app page root uses that class, and its scoped styles
// also reach this component's root element (and turned the grid into a flex box).
.query-workspace {
  // Fill what is left of the page below the heading; the graph takes whatever the builder doesn't.
  flex: 1;
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  grid-template-rows: max-content minmax(20rem, 1fr);
  grid-gap: 1rem;
  width: 100%;
}
</style>
