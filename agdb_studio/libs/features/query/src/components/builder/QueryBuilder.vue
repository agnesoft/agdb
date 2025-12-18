<script lang="ts" setup>
import type { TAB } from "../../composables/types";

defineProps<{
  tab: TAB;
}>();
</script>

<template>
  <div class="query-builder">
    <div
      class="query-input"
      :class="[tab]"
      contenteditable="true"
      role="textbox"
      aria-label="Query input"
      aria-multiline="false"
    ></div>
    <button
      type="button"
      class="button"
      :class="[tab === 'exec_mut' ? 'button-warning' : 'button-primary']"
    >
      Run query
    </button>
  </div>
</template>

<style lang="less" scoped>
.query-builder {
  width: 100%;
  height: 2rem;
  display: flex;
  button {
    border-top-left-radius: 0;
    border-bottom-left-radius: 0;
  }
}
.query-input {
  flex: 1;
  height: 100%;
  border: 1px solid var(--color-text);
  border-right: none;
  border-radius: 0.25rem 0 0 0.25rem;
  padding: 0 0.5rem;
  box-sizing: border-box;
  display: inline-block;
  line-height: 2rem;
  transition: background-color 0.4s ease;
  &.context {
    background-color: var(--color-background-soft);
  }
  &.exec_mut {
    background-color: var(--orange-background);
  }
}
</style>
