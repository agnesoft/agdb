<script lang="ts" setup>
import LoginForm from "../components/auth/LoginForm.vue";
</script>

<template>
    <main class="login-view">
        <div class="wrapper">
            <h1>Login</h1>
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

    h1 {
        margin-bottom: 0.6rem;
        padding-bottom: 0.3rem;
        border-bottom: 1px solid var(--color-text);
        flex-grow: 1;
    }
}
</style>
