<script lang="ts" setup></script>

<template>
  <div class="query-builder">
    <div
      class="query-input"
      contenteditable="true"
      role="textbox"
      aria-label="Query input"
      aria-multiline="false"
    ></div>
    <button type="button" class="button button-primary">Run query</button>
  </div>
</template>

<style lang="less" scoped>
.query-builder {
  width: 100%;
  height: 2rem;
  display: flex;
  gap: 0.5rem;
}
.query-input {
  flex: 1;
  height: 100%;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  padding: 0 0.5rem;
  box-sizing: border-box;
  display: inline-block;
  line-height: 2rem;
}
</style>
