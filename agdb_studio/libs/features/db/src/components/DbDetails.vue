<script lang="ts" setup>
import type { TRow } from "@agdb-studio/common/src/composables/table/types";
import type { DbUser, DbUserRole } from "@agnesoft/agdb_api/openapi";
import { computed, onMounted, ref, type PropType } from "vue";
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

const searchQuery = ref("");

const VISIBLE_COUNT = 10;
const expandedGroups = ref(new Set<DbUserRole>());

type RoleEntry = { key: DbUserRole; label: string };
const ROLE_ORDER: RoleEntry[] = [
  { key: "admin", label: "Admin" },
  { key: "write", label: "Read/Write" },
  { key: "read", label: "Read Only" },
];

const filteredUsers = computed(() => {
  const q = searchQuery.value.toLowerCase().trim();
  if (!q) return users.value ?? [];
  return (users.value ?? []).filter((u) =>
    u.username.toLowerCase().includes(q),
  );
});

type UserGroup = { role: DbUserRole; label: string; users: DbUser[] };

// Groups filtered users — hides empty groups as the search narrows results
const groupedUsers = computed<UserGroup[]>(() => {
  return ROLE_ORDER.map(({ key, label }) => ({
    role: key,
    label,
    users: filteredUsers.value.filter((u) => u.role === key),
  })).filter((g) => g.users.length > 0);
});

// Uses unfiltered users so the summary always shows the full picture
const roleCounts = computed(() => {
  const all = users.value ?? [];
  return ROLE_ORDER.map(({ key, label }) => ({
    label,
    count: all.filter((u) => u.role === key).length,
  })).filter((r) => r.count > 0);
});

const toggleGroup = (role: DbUserRole) => {
  if (expandedGroups.value.has(role)) {
    expandedGroups.value.delete(role);
  } else {
    expandedGroups.value.add(role);
  }
};

const visibleUsers = (group: UserGroup) => {
  if (expandedGroups.value.has(group.role)) return group.users;
  return group.users.slice(0, VISIBLE_COUNT);
};

const hiddenCount = (group: UserGroup) => {
  if (expandedGroups.value.has(group.role)) return 0;
  return Math.max(0, group.users.length - VISIBLE_COUNT);
};

const isCollapsible = (group: UserGroup) => {
  return group.users.length > VISIBLE_COUNT;
};
</script>

<template>
  <section class="db-details">
    <header>
      <h2>Database: {{ dbName }}</h2>
      <button
        v-if="canEditUsers"
        class="button button-bordered button-success add-button button-circle"
        title="Add user"
        type="button"
        @click="() => handleAddUser()"
      >
        <ChPlus class="add-icon" />
      </button>
    </header>

    <div class="db-users-toolbar">
      <input
        v-model="searchQuery"
        type="text"
        class="search-input"
        placeholder="Filter users..."
        aria-label="Filter users"
      />
      <span class="role-summary">
        <span v-for="r in roleCounts" :key="r.label" class="role-count">
          {{ r.count }} {{ r.label }}
        </span>
      </span>
    </div>

    <div v-for="group in groupedUsers" :key="group.role" class="role-group">
      <h3 class="role-group-header">
        {{ group.label }} ({{ group.users.length }})
      </h3>
      <ul class="db-users">
        <li
          v-for="user in visibleUsers(group)"
          :key="user.username"
          class="user-item"
        >
          <span
            class="username"
            :class="{
              clickable: !isOwner(user.username) && canEditUsers,
            }"
            @click="() => handleUsernameClick(user.username, user.role)"
            >{{ user.username }}</span
          >
          <button
            v-if="user.username !== dbParams.owner && canEditUsers"
            class="button button-bordered button-danger remove-button button-circle"
            title="Remove user"
            type="button"
            @click="handleRemoveUser(user.username)"
          >
            <ClCloseMd class="remove-icon" />
          </button>
        </li>
        <li v-if="isCollapsible(group)" class="show-more-item">
          <button
            class="button button-transparent show-more-button"
            type="button"
            @click="toggleGroup(group.role)"
          >
            {{
              hiddenCount(group) > 0
                ? `+${hiddenCount(group)} more`
                : "Show less"
            }}
          </button>
        </li>
      </ul>
    </div>
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
.db-users-toolbar {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-top: 0.5rem;
  margin-bottom: 0.5rem;
}
.search-input {
  padding: 0.3rem 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 0.25rem;
  background: var(--color-background);
  color: var(--color-text);
  font-size: 0.9rem;
  width: 200px;
  &:focus {
    outline: none;
    border-color: var(--color-border-hover);
  }
}
.role-summary {
  display: flex;
  gap: 0.25rem;
  color: var(--color-text-muted);
  font-size: 0.85rem;
}
.role-count {
  &:not(:last-child)::after {
    content: "·";
    margin-left: 0.25rem;
  }
}
.role-group {
  margin-bottom: 0.5rem;
}
.role-group-header {
  font-size: 0.9rem;
  font-weight: bold;
  color: var(--color-text-muted);
  padding: 0.25rem 0;
}
.db-users {
  list-style: none;
  padding: 0;
  margin-top: 0.25rem;
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
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
.remove-icon {
  color: var(--error-color);
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
.show-more-button {
  font-size: 0.85rem;
  color: var(--color-text-muted);
  &:hover {
    color: var(--color-text);
  }
}
</style>
