<template>
  <div class="language-icons">
    <NuxtLink to="/api-docs/rust">
      <img width="50" height="50" src="/images/rust.png" alt="rust" />
    </NuxtLink>
    <NuxtLink to="/api-docs/typescript">
      <img width="50" height="50" src="/images/ts.png" alt="ts" />
    </NuxtLink>
    <NuxtLink to="/api-docs/typescript">
      <img width="50" height="50" src="/images/js.png" alt="js" />
    </NuxtLink>
    <NuxtLink to="/api-docs/php">
      <img width="90" height="50" src="/images/php.svg" alt="php" />
    </NuxtLink>
    <img width="50" height="50" src="/images/python.png" alt="python" />
    <img width="50" height="50" src="/images/java.png" alt="java" />
    <img width="50" height="50" src="/images/c.png" alt="c" />
    <img width="50" height="50" src="/images/cpp.png" alt="cpp" />
    <img width="50" height="50" src="/images/csharp.png" alt="csharp" />
  </div>
</template>

<style scoped lang="less">
.language-icons {
  display: flex;
  flex-direction: row;
  gap: 1rem;
  align-items: center;
  margin: 2rem 0;
  width: 100%;
  justify-content: center;

  :deep(a) {
    display: inline-flex;
    align-items: center;
    text-decoration: none;
    transition: transform 0.2s;

    &:hover {
      transform: scale(1.1);
    }
  }
}
</style>
