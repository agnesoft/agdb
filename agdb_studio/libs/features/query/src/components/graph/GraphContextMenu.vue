<script lang="ts" setup>
import { computed, onMounted, onUnmounted, ref } from "vue";
import AgdbMenu from "@agdb-studio/common/src/components/menu/AgdbMenu.vue";
import type { Action } from "@agdb-studio/common/src/composables/content/types";

const props = withDefaults(
  defineProps<{
    x: number;
    y: number;
    title?: string;
    actions: Action<undefined>[];
  }>(),
  { title: "" },
);

const emit = defineEmits<{ close: [] }>();

const menuRef = ref<HTMLElement>();

/** Approximate menu size used to keep the menu inside the viewport. */
const MENU_WIDTH = 200;
const MENU_HEIGHT = 240;

const position = computed(() => ({
  left: `${Math.max(0, Math.min(props.x, window.innerWidth - MENU_WIDTH))}px`,
  top: `${Math.max(0, Math.min(props.y, window.innerHeight - MENU_HEIGHT))}px`,
}));

const onPointerDown = (event: Event) => {
  const inside =
    event.target instanceof Node && menuRef.value?.contains(event.target);
  if (!inside) {
    emit("close");
  }
};

const onKeydown = (event: KeyboardEvent) => {
  if (event.key === "Escape") {
    emit("close");
  }
};

onMounted(() => {
  document.addEventListener("pointerdown", onPointerDown, true);
  document.addEventListener("keydown", onKeydown);
});

onUnmounted(() => {
  document.removeEventListener("pointerdown", onPointerDown, true);
  document.removeEventListener("keydown", onKeydown);
});
</script>

<template>
  <Teleport to="body">
    <div
      ref="menuRef"
      class="graph-context-menu"
      role="menu"
      :style="position"
      data-testid="graph-context-menu"
      @contextmenu.prevent
    >
      <div v-if="props.title" class="menu-title">{{ props.title }}</div>
      <AgdbMenu :actions="props.actions" />
    </div>
  </Teleport>
</template>

<style lang="less" scoped>
.graph-context-menu {
  position: fixed;
  z-index: var(--z-index-dropdown);
}
.menu-title {
  font-size: 0.8rem;
  color: var(--color-text-muted);
  padding: 0 0.5rem 0.25rem;
}
</style>
