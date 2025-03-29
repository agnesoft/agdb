<script lang="ts" setup>
import { ref } from "vue";
import { useUserStore } from "@/composables/user/userStore";

const username = ref("");
const password = ref("");

const loading = ref(false);

const { addUser, fetchUsers } = useUserStore();

const add = (event: Event) => {
  loading.value = true;
  event.preventDefault();

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
    <form id="user-add-form" @submit="add">
      <div class="form-group">
        <label for="username">Username</label>
        <input id="username" v-model="username" type="text" />
      </div>
      <div class="form-group">
        <label for="password">Password</label>
        <input id="password" v-model="password" type="text" />
      </div>
      <button type="submit" class="button" @click="add">Add User</button>
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
