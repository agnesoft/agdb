<script lang="ts" setup>
import { computed } from "vue";
import { useRoute } from "vue-router";
import DashboardViewComponent from "@agdb-studio/dashboard/src/components/DashboardView.vue";

const route = useRoute();
const name = computed(() =>
  typeof route.params.name === "string" ? route.params.name : "",
);
</script>

<template>
  <div class="dashboard-page">
    <DashboardViewComponent :name="name" />
  </div>
</template>

<style lang="less" scoped>
.dashboard-page {
  width: 100%;
  box-sizing: border-box;
}
</style>
