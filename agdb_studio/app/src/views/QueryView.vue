<script lang="ts" setup>
import { useDbStore } from "@agdb-studio/db/src/composables/dbStore";
import { useRoute } from "vue-router";

import QueryViewComponent from "@agdb-studio/query/src/components/view/QueryView.vue";

const { getDbName } = useDbStore();

const route = useRoute();

const dbName = getDbName({
  owner: typeof route.params.owner === "string" ? route.params.owner : "",
  db: typeof route.params.db === "string" ? route.params.db : "",
});
</script>

<template>
  <div class="query-view">
    <h1>Database {{ dbName }} query</h1>
    <QueryViewComponent :dbName="dbName" />
  </div>
</template>

<style lang="less" scoped>
.query-view {
  width: 100%;
  height: 100%;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
</style>
