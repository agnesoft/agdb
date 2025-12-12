<script setup lang="ts">
import type { ContentNavigationItem } from "@nuxt/content";
import { useRoute } from "vue-router";

const navigation = inject<Ref<ContentNavigationItem[]>>("navigation");

const route = useRoute();
// Determine top-most folder from current path (e.g., "/docs/guides/..." -> "docs")
/* v8 ignore next -- @preserve */
const top = computed(() => route.path.split("/")[1] || "");

const filteredNavigation = computed<ContentNavigationItem[]>(() => {
  const items: ContentNavigationItem[] = navigation?.value ?? [];
  const topPath = `/${top.value}`;
  const section = items.find((i) => i.path === topPath);
  const children = section?.children ?? [];

  // Inject external links for the Examples section
  const mapped = children.map((child) => {
    if (child.path === "/docs/examples") {
      return {
        ...child,
        children: [
          ...(child.children || []),
          {
            title: "app_db",
            path: "https://github.com/agnesoft/agdb/tree/main/examples/app_db",
          },
          {
            title: "indexes",
            path: "https://github.com/agnesoft/agdb/tree/main/examples/indexes",
          },
          {
            title: "joins",
            path: "https://github.com/agnesoft/agdb/tree/main/examples/joins",
          },
          {
            title: "schema migration",
            path: "https://github.com/agnesoft/agdb/tree/main/examples/schema_migration",
          },
          {
            title: "server client - rust",
            path: "https://github.com/agnesoft/agdb/tree/main/examples/server_client_rust",
          },
          {
            title: "server client - typescript",
            path: "https://github.com/agnesoft/agdb/tree/main/examples/server_client_typescript",
          },
          {
            title: "strong types",
            path: "https://github.com/agnesoft/agdb/tree/main/examples/user_types",
          },
        ],
      };
    }
    return child;
  });

  return mapped;
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
