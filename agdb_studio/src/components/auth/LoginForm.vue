<script lang="ts" setup>
import { ref } from "vue";

const username = ref("");
const password = ref("");

const login = async () => {
    // todo - handle login
};
</script>

<template>
    <div>
        <h2>Login</h2>
        <form @submit.prevent="login">
            <div>
                <label for="username">Username:</label>
                <input type="text" id="username" v-model="username" required />
            </div>
            <div>
                <label for="password">Password:</label>
                <input type="password" id="password" v-model="password" required />
            </div>
            <button type="submit">Login</button>
        </form>
    </div>
</template>

<style lang="css" scoped></style>
