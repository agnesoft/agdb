<script setup lang="ts">
import GraphView2d from "@/components/graph/GraphView2d.vue";
import data from "@/tests/data/complexData.json";

const graphData = JSON.parse(JSON.stringify(data));
</script>

<template>
  <div class="main-wrapper">
    <GraphView2d :graph-data="graphData" />
  </div>
</template>

<style lang="less" scoped>
.main-wrapper {
  width: 100%;
  min-height: 100%;
  overflow: auto;
}
</style>
