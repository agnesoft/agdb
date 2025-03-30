<script lang="ts" setup>
import LogoIcon from "@/components/base/icons/LogoIcon.vue";
import LoginForm from "../components/auth/LoginForm.vue";
</script>

<template>
  <main class="login-view">
    <div class="wrapper">
      <h1><LogoIcon />login</h1>
      <LoginForm />
    </div>
  </main>
</template>

<style lang="less" scoped>
.login-view {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  --logo-icon-size: 60px;

  h1 {
    margin-bottom: 0.6rem;
    padding-bottom: 0.3rem;
    border-bottom: 1px solid var(--color-text);
    flex-grow: 1;
    display: flex;
    justify-content: flex-start;
    align-items: flex-end;
    gap: 1rem;
  }
}
</style>
