<script lang="ts" setup>
import { ref } from "vue";
import { useUserStore } from "../composables/userStore";

const username = ref("");
const password = ref("");

const loading = ref(false);

const { addUser, fetchUsers } = useUserStore();

const add = () => {
  if (!username.value || !password.value) return;
  loading.value = true;

  addUser({
    username: username.value,
    password: password.value,
  })
    .then(() => {
      loading.value = false;
      username.value = "";
      password.value = "";
      fetchUsers();
    })
    .catch(() => {
      loading.value = false;
    });
};
</script>

<template>
  <div class="user-add-form">
    <h2>Add User</h2>
    <form id="user-add-form" @submit.prevent="add">
      <div class="form-group">
        <label for="username">Username</label>
        <input
          id="username"
          v-model="username"
          type="text"
          required
          data-testid="username-input"
        />
      </div>
      <div class="form-group">
        <label for="password">Password</label>
        <input
          id="password"
          v-model="password"
          type="password"
          required
          data-testid="password-input"
        />
      </div>
      <button type="submit" class="button" data-testid="add-user-button">
        Add User
      </button>
    </form>
  </div>
</template>

<style lang="less" scoped>
.user-add-form {
  margin: 1rem auto;
  padding: 1rem;
  border: 1px solid var(--color-border);
  border-radius: 0.5rem;
}
.form-group {
  margin: 0.5rem 0;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}
</style>
