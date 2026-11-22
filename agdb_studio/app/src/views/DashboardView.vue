<script lang="ts" setup>
import { computed } from "vue";
import { useRoute } from "vue-router";
import DashboardViewComponent from "@agdb-studio/dashboard/src/components/DashboardView.vue";

const route = useRoute();
const param = (key: "owner" | "name") => {
  const value = route.params[key];
  return typeof value === "string" ? value : "";
};
const owner = computed(() => param("owner"));
const name = computed(() => param("name"));
</script>

<template>
  <div class="dashboard-page">
    <!-- The admin and user pages differ in access: never reuse one for the other. -->
    <DashboardViewComponent
      :key="String(route.name)"
      :owner="owner"
      :name="name"
    />
  </div>
</template>

<style lang="less" scoped>
.dashboard-page {
  width: 100%;
  box-sizing: border-box;
}
</style>
