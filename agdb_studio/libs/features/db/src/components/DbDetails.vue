<script lang="ts" setup>
import type { TRow } from "@agdb-studio/common/src/composables/table/types";
import { computed, onMounted, type PropType } from "vue";
import { useDbUsersStore } from "../composables/dbUsersStore";
import { ClCloseMd, ChPlus } from "@kalimahapps/vue-icons";
import { useDbDetails, type DbDetailsParams } from "../composables/dbDetails";

const props = defineProps({
  row: {
    type: Object as PropType<TRow>,
    required: false,
    default: undefined,
  },
});

const { fetchDbUsers, isDbRoleType } = useDbUsersStore();

const dbParams = computed<DbDetailsParams>(() => {
  return {
    owner: typeof props.row?.owner === "string" ? props.row?.owner : "",
    db: typeof props.row?.db === "string" ? props.row?.db : "",
    role:
      typeof props.row?.role === "string" && isDbRoleType(props.row.role)
        ? props.row?.role
        : "read",
  };
});

const {
  users,
  dbName,
  canEditUsers,
  handleRemoveUser,
  handleAddUser,
  isOwner,
  handleUsernameClick,
} = useDbDetails(dbParams);

onMounted(() => {
  fetchDbUsers(dbParams.value);
});
</script>

<template>
  <section class="db-details">
    <header>
      <h2>Database: {{ dbName }}</h2>
      <button
        v-if="canEditUsers"
        class="button button-bordered button-success add-button"
        title="Add user"
        type="button"
        @click="() => handleAddUser()"
      >
        <ChPlus class="add-icon" />
      </button>
    </header>

    <ul class="db-users">
      <li v-for="user in users" :key="user.username" class="user-item">
        <span
          class="username"
          :class="{
            clickable: !isOwner(user.username) && canEditUsers,
          }"
          @click="() => handleUsernameClick(user.username, user.role)"
          >{{ user.username }}</span
        >
        <span class="role">
          ({{ user.role.charAt(0).toLocaleUpperCase() }})
        </span>
        <button
          v-if="user.username !== dbParams.owner && canEditUsers"
          class="button button-bordered button-danger remove-button"
          title="Remove user"
          type="button"
          @click="handleRemoveUser(user.username)"
        >
          <ClCloseMd class="remove-icon" />
        </button>
      </li>
    </ul>
  </section>
</template>

<style lang="less" scoped>
.db-details {
  padding: 1rem;
  text-align: left;
  header {
    font-weight: bold;
    font-size: 1.05rem;
    border-bottom: 1px solid var(--color-border);
    padding-bottom: 0.5rem;
    display: flex;
    align-items: center;
  }
}
.db-users {
  list-style: none;
  padding: 0;
  margin-top: 0.5rem;
  display: flex;
  gap: 1rem;
  align-items: center;
  li {
    display: flex;
    align-items: center;
    gap: 0.2rem;
  }
  button {
    margin: 0 0.5rem;
  }
}
.role {
  color: var(--color-text-muted);
  margin-left: 0.2rem;
}
.remove-icon {
  color: var(--red);
}
.add-button {
  margin: 0 1rem;
}
.add-icon {
  color: var(--green);
  font-size: 1.5rem;
}
.username {
  &.clickable {
    cursor: pointer;
    transition: opacity 0.3s ease;
    &:hover {
      opacity: 0.8;
    }
  }
}
</style>
