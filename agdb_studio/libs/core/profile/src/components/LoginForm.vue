<script lang="ts" setup>
import { ref } from "vue";
import { useAuth } from "@agdb-studio/auth/src/auth";
import { getRouter } from "@agdb-studio/router/src/router";
import SpinnerIcon from "@agdb-studio/design/src/components/icons/SpinnerIcon.vue";

const { login } = useAuth();

const username = ref("");
const password = ref("");
const cluster = ref(false);

const loading = ref(false);
const error = ref("");

const clearError = () => {
  error.value = "";
};

const onLogin = async () => {
  loading.value = true;
  clearError();
  login({
    username: username.value,
    password: password.value,
    cluster: cluster.value,
  })
    .then(async () => {
      console.log("Login successful");
      const router = getRouter();
      const redirect = router.currentRoute.value.query.redirect;
      await router.push(
        typeof redirect === "string" ? redirect : { name: "home" },
      );
      loading.value = false;
    })
    .catch((e) => {
      console.error("Login failed:", e);
      loading.value = false;
      if (e.response && e.response.status === 401) {
        error.value = "Invalid username or password.";
      } else {
        error.value = "An unexpected error occurred. Please try again later.";
      }
      password.value = ""; // Clear password field on error
    });
};
</script>

<template>
  <div class="base-form login-form">
    <form @submit.prevent="onLogin">
      <div>
        <label for="username">Username:</label>
        <input
          id="username"
          v-model="username"
          type="text"
          required
          data-testid="inputUsername"
          autocomplete="on"
        />
      </div>
      <div>
        <label for="password">Password:</label>
        <input
          id="password"
          v-model="password"
          type="password"
          required
          data-testid="inputPassword"
          autocomplete="on"
        />
      </div>
      <div class="cluster-login">
        <input id="cluster-login" v-model="cluster" type="checkbox" />
        <label for="cluster-login">Login in all nodes in the cluster</label>
      </div>
      <button
        type="submit"
        class="button button-success button-lg"
        data-testid="buttonLogin"
      >
        <SpinnerIcon v-if="loading" />
        Login
      </button>
      <div v-if="error" class="error-message" data-testid="errorMessage">
        {{ error }}
      </div>
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
  .button {
    position: relative;
    .spinner-icon {
      position: absolute;
      left: 0.6em;
    }
  }
}
.cluster-login {
  display: flex;
  align-items: center;
  margin-top: 0.5rem;
  label {
    margin: 0;
    margin-left: 0.5rem;
  }
}
.error-message {
  color: red;
  margin-top: 0.5rem;
  font-size: 0.9rem;
}
</style>
