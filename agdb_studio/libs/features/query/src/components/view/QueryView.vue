<script lang="ts" setup>
import QueryBuilderTabs from "../builder/QueryBuilderTabs.vue";
import QueryGraph from "../graph/QueryGraph.vue";
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
