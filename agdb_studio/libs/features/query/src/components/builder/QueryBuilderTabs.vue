<script lang="ts" setup>
import { provide, ref } from "vue";
import QueryBuilder from "./QueryBuilder.vue";
import { type TAB, TABS } from "../../composables/types";

const activeTab = ref<TAB>(TABS[0]);

const tabs = ref<HTMLElement[]>();

const handleKeydown = (event: KeyboardEvent) => {
  const currentIndex = TABS.indexOf(activeTab.value);
  if (event.key === "ArrowRight") {
    const nextIndex = (currentIndex + 1) % TABS.length;

    /* v8 ignore next -- @preserve */
    activeTab.value = TABS[nextIndex] ?? activeTab.value;
    event.preventDefault();
  } else if (event.key === "ArrowLeft") {
    const prevIndex = (currentIndex - 1 + TABS.length) % TABS.length;

    /* v8 ignore next -- @preserve */
    activeTab.value = TABS[prevIndex] ?? activeTab.value;
    event.preventDefault();
  }
  tabs.value?.[TABS.indexOf(activeTab.value)]?.focus();
};

provide("activeTab", activeTab);
</script>

<template>
  <div class="query-builder-tabs" @keydown="handleKeydown">
    <div class="tabs" role="tablist">
      <button
        v-for="tab in TABS"
        :id="`query-tabpanel-${tab}`"
        :key="tab"
        ref="tabs"
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
    <QueryBuilder :aria-controls="`query-tabpanel-${activeTab}`" />
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
