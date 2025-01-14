<script lang="ts" setup>
import { useAccount } from "@/composables/user/account";
import AgdbDropdown from "../base/dropdown/AgdbDropdown.vue";
import { ClUser02 } from "@kalimahapps/vue-icons";
import { useAuth } from "@/composables/user/auth";
import AgdbMenu from "../base/menu/AgdbMenu.vue";
import useModal from "@/composables/modal/modal";

const { logout } = useAuth();
const { username } = useAccount();
const { openModal } = useModal();

const actions = [
    {
        key: "change-password",
        label: "Change password",
        action: () => {
            openModal({
                header: "Change password",
                content: [
                    {
                        input: {
                            type: "password",
                            label: "Current password",
                            key: "currentPassword",
                            required: true,
                        },
                    },
                    {
                        input: {
                            type: "password",
                            label: "New password",
                            key: "newPassword",
                            required: true,
                        },
                    },
                    {
                        input: {
                            type: "password",
                            label: "Confirm new password",
                            key: "confirmNewPassword",
                            required: true,
                        },
                    },
                ],
            });
        },
        onConfirm: () => {
            console.log("Change password");
        },
    },
    {
        key: "logout",
        label: "Logout",
        action: () => {
            logout();
        },
    },
];
</script>

<template>
    <AgdbDropdown>
        <template v-slot:trigger>
            <div class="user-dropdown-trigger button">
                <ClUser02 />
                <span>{{ username }}</span>
            </div>
        </template>
        <template v-slot:content>
            <div class="user-dropdown-content">
                <AgdbMenu :actions="actions" />
            </div>
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
