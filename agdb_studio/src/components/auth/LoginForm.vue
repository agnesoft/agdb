<script lang="ts" setup>
import { ref } from "vue";

const username = ref("");
const password = ref("");

const login = async () => {
    // todo - handle login
};
</script>

<template>
    <div class="base-form login-form">
        <form @submit.prevent="login">
            <div>
                <label for="username">Username:</label>
                <input type="text" id="username" v-model="username" required />
            </div>
            <div>
                <label for="password">Password:</label>
                <input
                    type="password"
                    id="password"
                    v-model="password"
                    required
                />
            </div>
            <button type="submit" class="button button-success">Login</button>
        </form>
    </div>
</template>

<style lang="less" scoped>
.login-form {
    width: 300px;
    [type="submit"] {
        width: 100%;
        font-size: 1.2rem;
        margin-top: 0.6rem;
    }
}
</style>
