<script lang="ts" setup>
import { ref } from "vue";
import QueryBuilder from "./QueryBuilder.vue";
import { type TAB, TABS } from "../../composables/types";

const activeTab = ref<TAB>(TABS[0]);
</script>

<template>
  <div class="query-builder-tabs">
    <div class="tabs">
      <button
        v-for="tab in TABS"
        :key="tab"
        class="button button-tab"
        :class="{ active: activeTab === tab }"
        type="button"
        @click="activeTab = tab"
      >
        {{ tab === "context" ? "exec (Context)" : tab }}
      </button>
    </div>
    <QueryBuilder :tab="activeTab" />
  </div>
</template>

<style lang="less" scoped>
.tabs {
  display: flex;
  margin-left: 1.5rem;
  gap: 0.5rem;
}
</style>
