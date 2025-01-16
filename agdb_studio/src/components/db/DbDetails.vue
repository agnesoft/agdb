<script lang="ts" setup>
import type { TRow } from "@/composables/table/types";
import { computed, onMounted, type PropType } from "vue";
import { useDbUsersStore } from "@/composables/db/dbUsersStore";
import { ClCloseMd, ChPlus } from "@kalimahapps/vue-icons";
import { useDbDetails, type DbDetailsParams } from "@/composables/db/dbDetails";
import type { DbUserRole } from "agdb_api/dist/openapi";

const props = defineProps({
    row: {
        type: Object as PropType<TRow>,
        required: false,
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

const { users, dbName, canEditUsers, handleRemoveUser, handleAddUser } =
    useDbDetails(dbParams);

onMounted(() => {
    fetchDbUsers(dbParams.value);
});

const isOwner = (username: string) => {
    return username === dbParams.value.owner;
};

const handleUsernameClick = (username: string, role: DbUserRole) => {
    if (isOwner(username) || !canEditUsers.value) {
        return;
    }
    handleAddUser({ username, db_role: role });
};
</script>

<template>
    <section class="db-details">
        <header>
            <h2>Database: {{ dbName }}</h2>
            <button
                v-if="canEditUsers"
                class="button button-transparent add-button"
                title="Add user"
                @click="() => handleAddUser()"
            >
                <ChPlus class="add-icon" />
            </button>
        </header>

        <ul class="db-users">
            <li v-for="user in users" :key="user.username" class="user-item">
                <span
                    class="username"
                    @click="() => handleUsernameClick(user.username, user.role)"
                    :class="{
                        clickable: !isOwner(user.username) && canEditUsers,
                    }"
                    >{{ user.username }}</span
                >
                <span class="role">
                    ({{ user.role.charAt(0).toLocaleUpperCase() }})
                </span>
                <button
                    v-if="user.username !== dbParams.owner && canEditUsers"
                    class="button button-transparent remove-button"
                    @click="handleRemoveUser(user.username)"
                    title="Remove user"
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
    }
}
.db-users {
    list-style: none;
    padding: 0;
    margin-top: 0.5rem;
    display: flex;
    gap: 1rem;
    li {
        display: flex;
        align-items: center;
        gap: 0.2rem;
    }
    button {
        padding: 0;
        height: 1em;
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
    height: 1em;
    padding: 0 1rem;
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
