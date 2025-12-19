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
  display: grid;
  grid-template-columns: 1fr;
  grid-template-rows: max-content 1fr;
  grid-gap: 1rem;
  width: 100%;
  height: 100%;
}
</style>
