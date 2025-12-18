<script lang="ts" setup>
import { ref } from "vue";
import QueryBuilder from "./QueryBuilder.vue";
import { type TAB, TABS } from "../../composables/types";

const activeTab = ref<TAB>(TABS[0]);
</script>

<template>
  <div class="query-builder-tabs">
    <div class="tabs" role="tablist">
      <button
        v-for="tab in TABS"
        :key="tab"
        class="button button-tab"
        :class="{ active: activeTab === tab }"
        type="button"
        @click="activeTab = tab"
        role="tab"
        :aria-selected="activeTab === tab"
        :id="`query-tabpanel-${tab}`"
        :tabindex="activeTab === tab ? 0 : -1"
      >
        {{ tab === "context" ? "exec (Context)" : tab }}
      </button>
    </div>
    <QueryBuilder
      :tab="activeTab"
      :aria-controls="`query-tabpanel-${activeTab}`"
    />
  </div>
</template>

<style lang="less" scoped>
.tabs {
  display: flex;
  margin-left: 1.5rem;
  gap: 0.5rem;
}
</style>
