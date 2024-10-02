<script lang="ts" setup>
import { ref } from "vue";
import { login } from "@/services/auth.service";

const username = ref("");
const password = ref("");

const loading = ref(false);
const error = ref("");

const clearError = () => {
    error.value = "";
};

const onLogin = async () => {
    loading.value = true;
    clearError();
    login(username.value, password.value)
        .then(() => {
            loading.value = false;
        })
        .catch((e) => {
            loading.value = false;
            error.value = e.message;
        });
};
</script>

<template>
    <div class="base-form login-form">
        <form @submit.prevent="onLogin">
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
    input {
        border-radius: 0.2rem;
    }
}
</style>
