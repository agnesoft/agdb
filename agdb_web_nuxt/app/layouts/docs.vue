<script setup lang="ts">
import type { ContentNavigationItem } from "@nuxt/content";
import { useRoute } from "vue-router";

const navigation = inject<Ref<ContentNavigationItem[]>>("navigation");

console.log("Full Navigation:", navigation?.value);

const route = useRoute();
// Determine top-most folder from current path (e.g., "/docs/guides/..." -> "docs")
const top = computed(() => route.path.split("/")[1] || "");

const filteredNavigation = computed<ContentNavigationItem[]>(() => {
  const items: ContentNavigationItem[] = navigation?.value ?? [];
  const topPath = `/${top.value}`;
  const section = items.find((i) => i.path === topPath);
  const children = section?.children ?? [];
  return children;
});
</script>

<template>
  <UContainer>
    <UPage>
      <template #left>
        <UPageAside>
          <UContentNavigation highlight :navigation="filteredNavigation" />
        </UPageAside>
      </template>

      <slot></slot>
    </UPage>
  </UContainer>
</template>
