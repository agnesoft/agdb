<script lang="ts" setup>
import { useAccount } from "@/composables/profile/account";
import AgdbDropdown from "../base/dropdown/AgdbDropdown.vue";
import { ClUser02 } from "@kalimahapps/vue-icons";
import ProfileMenu from "./ProfileMenu.vue";

const { username } = useAccount();
</script>

<template>
    <AgdbDropdown>
        <template #trigger>
            <div class="user-dropdown-trigger button">
                <ClUser02 />
                <span>{{ username }}</span>
            </div>
        </template>
        <template #content>
            <ProfileMenu />
        </template>
    </AgdbDropdown>
</template>

<style lang="less" scoped>
.user-dropdown-trigger {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    max-width: 12rem;
    overflow: hidden;
    white-space: nowrap;
    font-size: 0.9rem;
}
</style>
