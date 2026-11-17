<script lang="ts" setup>
import { computed, provide, ref } from "vue";
import QueryBuilder from "./QueryBuilder.vue";
import { type TAB, TABS } from "../../composables/types";

const props = defineProps<{ tabs?: readonly TAB[] }>();
const shownTabs = computed(() => props.tabs ?? TABS);

const activeTab = ref<TAB>(shownTabs.value[0] ?? TABS[0]);

const tabRefs = ref<HTMLElement[]>();

const handleKeydown = (event: KeyboardEvent) => {
  const tabList = shownTabs.value;
  const currentIndex = tabList.indexOf(activeTab.value);
  if (event.key === "ArrowRight") {
    const nextIndex = (currentIndex + 1) % tabList.length;

    /* v8 ignore next -- @preserve */
    activeTab.value = tabList[nextIndex] ?? activeTab.value;
    event.preventDefault();
    tabRefs.value?.[tabList.indexOf(activeTab.value)]?.focus();
  } else if (event.key === "ArrowLeft") {
    const prevIndex = (currentIndex - 1 + tabList.length) % tabList.length;

    /* v8 ignore next -- @preserve */
    activeTab.value = tabList[prevIndex] ?? activeTab.value;
    event.preventDefault();
    tabRefs.value?.[tabList.indexOf(activeTab.value)]?.focus();
  }
};

provide("activeTab", activeTab);
</script>

<template>
  <div class="query-builder-tabs" @keydown="handleKeydown">
    <div class="tabs" role="tablist">
      <button
        v-for="tab in shownTabs"
        :id="`query-tabpanel-${tab}`"
        :key="tab"
        ref="tabRefs"
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
