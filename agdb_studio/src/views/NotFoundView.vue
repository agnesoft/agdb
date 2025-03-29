<script lang="ts" setup></script>

<template>
  <div class="not-found">
    <h1>404</h1>
    <p>Page not found</p>
  </div>
</template>

<style lang="less" scoped>
.not-found {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: calc(100vh - 10rem);
  min-height: 10rem;
  font-family: "Arial", sans-serif;
  h1 {
    font-size: 6rem;
    margin: 0;
    border-bottom: 2px solid var(--color-border);
  }

  p {
    font-size: 1.5rem;
    margin: 1rem;
  }
}
</style>
