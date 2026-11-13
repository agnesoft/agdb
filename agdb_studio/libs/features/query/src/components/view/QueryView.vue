<script lang="ts" setup>
import { onMounted, provide, ref } from "vue";
import QueryBuilderTabs from "../builder/QueryBuilderTabs.vue";
import QueryGraph from "../graph/QueryGraph.vue";
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
  <div class="query-view">
    <QueryBuilderTabs />
    <QueryGraph />
  </div>
</template>

<style lang="less" scoped>
.query-view {
  // Fill what is left of the page below the heading; the graph takes whatever the builder doesn't.
  flex: 1;
  display: grid;
  grid-template-columns: 1fr;
  grid-template-rows: max-content minmax(20rem, 1fr);
  grid-gap: 1rem;
  width: 100%;
}
</style>
