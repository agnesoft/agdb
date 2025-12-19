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
        :id="`query-tabpanel-${tab}`"
        :key="tab"
        class="button button-tab query-tab"
        :class="{ active: activeTab === tab }"
        type="button"
        role="tab"
        :aria-selected="activeTab === tab"
        :tabindex="activeTab === tab ? 0 : -1"
        @click="activeTab = tab"
      >
        {{ tab }}
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
  margin-inline: 1.5rem;
  gap: 0.5rem;
}
.query-tab {
  &:last-of-type {
    margin-left: auto;
  }
}
</style>
