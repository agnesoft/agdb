<script lang="ts" setup>
import type { TRow } from "@/composables/table/types";
import { computed, onMounted, type PropType } from "vue";
import { useDbUsers } from "@/composables/db/dbUsers";
import { ClCloseMd } from "@kalimahapps/vue-icons";

const props = defineProps({
    row: {
        type: Object as PropType<TRow>,
        required: false,
    },
});

const { getDbUsers, fetchDbUsers, removeUser } = useDbUsers();

const owner = computed(() => {
    return typeof props.row?.owner === "string" ? props.row?.owner : "";
});

const db = computed(() => {
    return typeof props.row?.db === "string" ? props.row?.db : "";
});

const role = computed(() => {
    return typeof props.row?.role === "string" ? props.row?.role : "";
});

const users = computed(() => {
    return getDbUsers({ owner: owner.value, db: db.value });
});
onMounted(() => {
    fetchDbUsers({ owner: owner.value, db: db.value });
});
</script>

<template>
    <div class="db-details">
        <h2>Users</h2>
        <ul class="db-users">
            <li v-for="user in users" :key="user.user">
                <span class="username">{{ user.user }}</span>
                <span class="role">
                    ({{ user.role.charAt(0).toLocaleUpperCase() }})
                </span>
                <button
                    v-if="user.user !== owner && role === 'admin'"
                    class="button button-transparent"
                    @click="removeUser({ username: user.user, owner, db })"
                    title="Remove user"
                >
                    <ClCloseMd class="remove-icon" />
                </button>
            </li>
        </ul>
    </div>
</template>

<style lang="less" scoped>
.db-details {
    padding: 1rem;
    text-align: left;
    h2 {
        font-weight: bold;
        font-size: 1.05rem;
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
</style>
