<script lang="ts" setup>
import { computed, onMounted, ref } from "vue";
import type { DbUser, DbUserRole } from "@agnesoft/agdb_api";
import { checkClient, client } from "@agdb-studio/api/src/api";
import useModal from "@agdb-studio/common/src/composables/modal/modal";
import { useContentInputs } from "@agdb-studio/common/src/composables/content/inputs";
import { KEY_MODAL } from "@agdb-studio/common/src/composables/modal/constants";
import { EMPHASIZED_CLASSNAME } from "@agdb-studio/common/src/composables/content/utils";
import { addNotification } from "@agdb-studio/notification/src/composables/notificationStore";
import RoleUsers from "@agdb-studio/db/src/components/RoleUsers.vue";
import { useDbUsersStore } from "@agdb-studio/db/src/composables/dbUsersStore";
import { useAccount } from "@agdb-studio/auth/src/account";
import {
  adminMode,
  canShare,
  useDashboardStore,
  type DashboardKey,
} from "../composables/dashboardStore";

/** Who the dashboard is shared with; admins add, change and remove users. */
const props = defineProps<DashboardKey & { role: DbUserRole }>();

const { openModal } = useModal();
const { getInputValue } = useContentInputs();
const { isDbRoleType } = useDbUsersStore();
const { username: me } = useAccount();
const store = useDashboardStore();

const users = ref<DbUser[]>();
const canEdit = computed(() => canShare(props.role));
const label = computed(() => `${props.owner}/${props.name}`);
const key = () => ({ owner: props.owner, name: props.name });

const fetchUsers = async () => {
  checkClient(client);
  const response = adminMode()
    ? await client.value.admin_dashboard_user_list(key())
    : await client.value.dashboard_user_list(key());
  users.value = response.data;
};

onMounted(async () => {
  try {
    await fetchUsers();
  } catch {
    // The API error interceptor shows a notification.
  }
});

const addUser = (username?: string, role?: DbUserRole) =>
  openModal({
    header: "Share dashboard",
    content: [
      {
        paragraph: [
          { text: "Add/change user role in the dashboard " },
          { text: label.value, className: EMPHASIZED_CLASSNAME },
          {
            text: ". Widgets run their queries with each viewer's own database access.",
          },
        ],
      },
      {
        input: {
          key: "username",
          label: "Username",
          type: "text",
          autofocus: true,
          required: true,
          value: username,
        },
      },
      {
        input: {
          key: "role",
          label: "Role",
          type: "select",
          options: [
            { value: "admin", label: "Admin" },
            { value: "write", label: "Read/Write" },
            { value: "read", label: "Read Only" },
          ],
          value: role ?? "read",
          required: true,
        },
      },
    ],
    onConfirm: async () => {
      const username = String(
        getInputValue(KEY_MODAL, "username") ?? "",
      ).trim();
      const db_role = String(getInputValue(KEY_MODAL, "role") ?? "");
      if (!username || !isDbRoleType(db_role)) return false;
      checkClient(client);
      const params = { ...key(), username, db_role };
      if (adminMode()) await client.value.admin_dashboard_user_add(params);
      else await client.value.dashboard_user_add(params);
      addNotification({
        type: "success",
        title: "User added/changed",
        message: `User ${username} can now use the dashboard ${label.value} (${db_role}).`,
      });
      // A changed own role changes what the list and this panel allow.
      if (!adminMode() && username === me.value) await store.fetchDashboards();
      await fetchUsers();
      return true;
    },
  });

const removeUser = (username: string) =>
  openModal({
    header: "Remove user",
    content: [
      {
        paragraph: [
          { text: "Remove user " },
          { text: username, className: EMPHASIZED_CLASSNAME },
          { text: " from dashboard " },
          { text: label.value, className: EMPHASIZED_CLASSNAME },
          { text: "?" },
        ],
      },
    ],
    onConfirm: async () => {
      if (!adminMode() && username === me.value) {
        // Leaving: the store also syncs the list and the home dashboard.
        await store.leaveDashboard(key());
        return true;
      }
      checkClient(client);
      const params = { ...key(), username };
      if (adminMode()) await client.value.admin_dashboard_user_remove(params);
      else await client.value.dashboard_user_remove(params);
      await fetchUsers();
      return true;
    },
  });

const editUser = (username: string, role: DbUserRole) => {
  if (username === props.owner || !canEdit.value) return;
  addUser(username, role);
};
</script>

<template>
  <RoleUsers
    :title="`Dashboard: ${label}`"
    :users="users"
    :owner="owner"
    :can-edit="canEdit"
    data-testid="dashboard-users"
    @add="addUser()"
    @edit="editUser"
    @remove="removeUser"
  />
</template>
