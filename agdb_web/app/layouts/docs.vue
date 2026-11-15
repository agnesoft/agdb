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
            path: "https://github.com/agnesoft/agdb/blob/main/agdb/examples/app_db.rs",
          },
          {
            title: "indexes",
            path: "https://github.com/agnesoft/agdb/blob/main/agdb/examples/indexes.rs",
          },
          {
            title: "joins",
            path: "https://github.com/agnesoft/agdb/blob/main/agdb/examples/joins.rs",
          },
          {
            title: "schema migration",
            path: "https://github.com/agnesoft/agdb/blob/main/agdb/examples/schema_migration.rs",
          },
          {
            title: "server client - rust",
            path: "https://github.com/agnesoft/agdb/blob/main/agdb_api/examples/server_client.rs",
          },
          {
            title: "server client - typescript",
            path: "https://github.com/agnesoft/agdb/tree/main/agdb_api/typescript/examples/server_client",
          },
          {
            title: "strong types",
            path: "https://github.com/agnesoft/agdb/blob/main/agdb/examples/user_types.rs",
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

      <slot />
    </UPage>
  </UContainer>
</template>
