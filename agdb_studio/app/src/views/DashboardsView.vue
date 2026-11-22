<script lang="ts" setup>
import { useRoute } from "vue-router";
import DashboardList from "@agdb-studio/dashboard/src/components/DashboardList.vue";

const route = useRoute();
</script>

<template>
  <!-- The admin and user pages list different dashboards: never reuse one for the other. -->
  <DashboardList :key="String(route.name)" />
</template>
