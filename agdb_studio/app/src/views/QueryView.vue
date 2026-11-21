<script lang="ts" setup>
import { useDbStore } from "@agdb-studio/db/src/composables/dbStore";
import { computed, useTemplateRef } from "vue";
import { useRoute } from "vue-router";

import QueryViewComponent from "@agdb-studio/query/src/components/view/QueryView.vue";
import PinToDashboardButton from "@agdb-studio/dashboard/src/components/PinToDashboardButton.vue";

const { getDbName } = useDbStore();

const route = useRoute();

const owner = typeof route.params.owner === "string" ? route.params.owner : "";
const db = typeof route.params.db === "string" ? route.params.db : "";
const dbName = getDbName({ owner, db });

// The pin button reads the builder's query store entry (rows, colours, view).
const queryView = useTemplateRef("queryView");
const queryId = computed(() => queryView.value?.queryId);
</script>

<template>
  <div class="query-view">
    <div class="heading">
      <h1>Database {{ dbName }} query</h1>
      <PinToDashboardButton
        v-if="route.name === 'query'"
        :owner="owner"
        :db="db"
        :query-id="queryId"
      />
    </div>
    <QueryViewComponent ref="queryView" />
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
.heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  h1 {
    margin: 0;
  }
}
</style>
