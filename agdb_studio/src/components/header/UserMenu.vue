<script lang="ts" setup>
import { useAuth } from "@/composables/user/auth";
import AgdbMenu from "../base/menu/AgdbMenu.vue";
import useModal from "@/composables/modal/modal";
import { client } from "@/services/api.service";
import { useContentInputs } from "@/composables/content/inputs";
import { KEY_MODAL } from "@/composables/modal/constants";
import { convertArrayOfStringsToContent } from "@/composables/content/utils";

const { logout } = useAuth();
const { openModal } = useModal();
const { getInputValue } = useContentInputs();

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
                            rules: [
                                (value: string) => {
                                    if (value.length < 8) {
                                        return "Password must be at least 8 characters long";
                                    }
                                    return undefined;
                                },
                            ],
                        },
                    },
                    {
                        input: {
                            type: "password",
                            label: "Confirm new password",
                            key: "confirmNewPassword",
                            required: true,
                            rules: [
                                (value: string) => {
                                    if (
                                        value !==
                                        getInputValue<string>(
                                            KEY_MODAL,
                                            "newPassword",
                                        )
                                    ) {
                                        return "Passwords do not match";
                                    }
                                    return undefined;
                                },
                            ],
                        },
                    },
                ],
                onConfirm: async () => {
                    const currentPassword = getInputValue<string>(
                        KEY_MODAL,
                        "currentPassword",
                    );
                    const newPassword = getInputValue<string>(
                        KEY_MODAL,
                        "newPassword",
                    );
                    const confirmNewPassword = getInputValue(
                        KEY_MODAL,
                        "confirmNewPassword",
                    );
                    if (newPassword !== confirmNewPassword) {
                        return false;
                    }
                    const response = await client.value?.user_change_password(
                        null,
                        {
                            password: currentPassword,
                            new_password: newPassword,
                        },
                    );
                    return response ? true : false;
                },
            });
        },
    },
    {
        key: "logout",
        label: "Logout",
        action: () => {
            openModal({
                header: "Logout",
                content: convertArrayOfStringsToContent([
                    "Are you sure you want to logout?",
                ]),
                onConfirm: () => {
                    console.log("Logging out");
                    logout();
                    return true;
                },
            });
        },
    },
];
</script>
<template>
    <div class="user-dropdown-content">
        <AgdbMenu :actions="actions" />
    </div>
</template>
