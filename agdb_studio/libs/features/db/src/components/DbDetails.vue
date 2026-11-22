<script lang="ts" setup>
import type { TRow } from "@agdb-studio/common/src/composables/table/types";
import { computed, onMounted, type PropType } from "vue";
import { useDbUsersStore } from "../composables/dbUsersStore";
import { useDbDetails, type DbDetailsParams } from "../composables/dbDetails";
import RoleUsers from "./RoleUsers.vue";

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
  handleUsernameClick,
} = useDbDetails(dbParams);

onMounted(() => {
  fetchDbUsers(dbParams.value);
});
</script>

<template>
  <RoleUsers
    class="db-details"
    :title="`Database: ${dbName}`"
    :users="users"
    :owner="dbParams.owner"
    :can-edit="canEditUsers"
    @add="handleAddUser()"
    @edit="handleUsernameClick"
    @remove="handleRemoveUser"
  />
</template>
