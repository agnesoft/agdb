<script lang="ts" setup>
import type { TRow } from "@/composables/table/types";
import { computed, onMounted, type PropType } from "vue";
import { useDbUsers } from "@/composables/db/dbUsers";
import { ClCloseMd, ChPlus } from "@kalimahapps/vue-icons";
import useModal from "@/composables/modal/modal";
import { useDbStore } from "@/composables/db/dbStore";
import { EMPHESIZED_CLASSNAME } from "@/composables/content/utils";
import { KEY_MODAL } from "@/composables/modal/constants";
import { useContentInputs } from "@/composables/content/inputs";

const props = defineProps({
    row: {
        type: Object as PropType<TRow>,
        required: false,
    },
});

const { getDbUsers, fetchDbUsers, removeUser, addUser, isDbRoleType } =
    useDbUsers();
const { getInputValue } = useContentInputs();

const dbParams = computed(() => {
    return {
        owner: typeof props.row?.owner === "string" ? props.row?.owner : "",
        db: typeof props.row?.db === "string" ? props.row?.db : "",
        role: typeof props.row?.role === "string" ? props.row?.role : "",
    };
});

const users = computed(() => {
    return getDbUsers(dbParams.value);
});

const { openModal } = useModal();
const { getDbName } = useDbStore();

const dbName = computed(() => {
    return getDbName(dbParams.value);
});

const canEditUsers = computed(() => {
    return dbParams.value.role === "admin";
});

const handleRemoveUser = (username: string) => {
    openModal({
        header: "Remove user",
        content: [
            {
                paragraph: [
                    { text: "Are you sure you want to remove user " },
                    { text: username, className: EMPHESIZED_CLASSNAME },
                    { text: " from database " },
                    { text: dbName.value, className: EMPHESIZED_CLASSNAME },
                    { text: "?" },
                ],
            },
        ],

        onConfirm: () => {
            removeUser({
                owner: dbParams.value.owner,
                db: dbParams.value.db,
                username: username,
            }).then(() => {
                fetchDbUsers(dbParams.value);
            });
        },
    });
};

const handleAddUser = () => {
    openModal({
        header: "Add user",
        content: [
            {
                paragraph: [
                    { text: "Add user to database " },
                    { text: dbName.value, className: EMPHESIZED_CLASSNAME },
                ],
            },
            {
                input: {
                    key: "username",
                    label: "Username",
                    type: "text",
                    autofocus: true,
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
                    defaultValue: "write",
                },
            },
        ],
        onConfirm: () => {
            const username = getInputValue(KEY_MODAL, "username")?.toString();
            const db_role = getInputValue(KEY_MODAL, "role")?.toString();

            console.log({ username, db_role });
            if (username?.length && db_role && isDbRoleType(db_role)) {
                console.log("Adding user");
                addUser({ ...dbParams.value, username, db_role }).then(() => {
                    console.log("Fetching users");
                    fetchDbUsers(dbParams.value);
                });
            }
        },
    });
};

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
                class="button button-transparent add-button"
                title="Add user"
                @click="handleAddUser"
            >
                <ChPlus class="add-icon" />
            </button>
        </header>

        <ul class="db-users">
            <li v-for="user in users" :key="user.user">
                <span class="username">{{ user.user }}</span>
                <span class="role">
                    ({{ user.role.charAt(0).toLocaleUpperCase() }})
                </span>
                <button
                    v-if="user.user !== dbParams.owner && canEditUsers"
                    class="button button-transparent"
                    @click="handleRemoveUser(user.user)"
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
</style>
