<script lang="ts" setup>
import { useDbStore } from "@agdb-studio/db/src/composables/dbStore";
import { useRoute } from "vue-router";

const { getDbName } = useDbStore();

const route = useRoute();

const dbName = getDbName({
  owner: typeof route.params.owner === "string" ? route.params.owner : "",
  db: typeof route.params.db === "string" ? route.params.db : "",
});
</script>

<template>
  <div>
    <h1>Database {{ dbName }} query</h1>
    <div class="query-input">
      <input type="text" placeholder="Type your query here..." />
      <button class="button button-primary">Run Query</button>
    </div>
    <div class="query-results">
      <!-- Query results will be displayed here -->
    </div>
  </div>
</template>

<style lang="less" scoped></style>
