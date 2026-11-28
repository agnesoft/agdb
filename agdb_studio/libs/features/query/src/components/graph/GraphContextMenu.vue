<script lang="ts" setup>
import { computed, onMounted, onUnmounted, ref } from "vue";
import AgdbMenu from "@agdb-studio/common/src/components/menu/AgdbMenu.vue";
import type { Action } from "@agdb-studio/common/src/composables/content/types";
import type { PaletteSlot } from "../../utils/graphColorRules";
import ColorSwatchPicker from "./ColorSwatchPicker.vue";

const props = withDefaults(
  defineProps<{
    x: number;
    y: number;
    title?: string;
    actions: Action<undefined>[];
    /** When given, a row of swatches sets the element's own colour. */
    palette?: readonly string[];
    color?: PaletteSlot;
  }>(),
  { title: "", palette: undefined, color: undefined },
);

const emit = defineEmits<{
  close: [];
  color: [slot: PaletteSlot | undefined];
}>();

const pickColor = (slot: PaletteSlot | undefined) => {
  emit("color", slot);
  emit("close");
};

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
      <div
        v-if="props.palette?.length"
        class="menu-colors"
        data-testid="menu-colors"
      >
        <span class="menu-colors-label">Colour</span>
        <ColorSwatchPicker
          inline
          :palette="props.palette"
          :model-value="props.color"
          label="Colour of this element"
          @update:model-value="pickColor"
        />
      </div>
    </div>
  </Teleport>
</template>

<style lang="less" scoped>
.graph-context-menu {
  position: fixed;
  z-index: var(--z-index-context-menu);
}
.menu-colors {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-top: 0.25rem;
  padding: 0.35rem 0.5rem;
  font-size: 0.8rem;
  border-radius: 0.3rem;
  background-color: var(--color-background-mute);
  border: 1px solid var(--color-border);
}
.menu-colors-label {
  color: var(--color-text-muted);
}
.menu-title {
  font-size: 0.8rem;
  color: var(--color-text-muted);
  padding: 0 0.5rem 0.25rem;
}
</style>
