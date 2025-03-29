import type { UserStatus } from "@agnesoft/agdb_api/openapi";
import { useContentInputs } from "../content/inputs";
import { client } from "@/services/api.service";
import { KEY_MODAL } from "../modal/constants";
import { addNotification } from "../../../libs/notification/src/composables/notificationStore";
import { convertArrayOfStringsToContent } from "../content/utils";
import type { Column, TRow } from "../table/types";

const { getInputValue } = useContentInputs();

const userActions: Action<UserStatus>[] = [
    {
        key: "change_password",
        label: "Change Password",
        action: async ({ params }) => {
            const password = getInputValue<string>(
                KEY_MODAL,
                "password",
            )?.toString();
            return client.value
                ?.admin_user_change_password(
                    {
                        username: params.username,
                    },
                    { password },
                )
                .then(() => {
                    addNotification({
                        type: "success",
                        title: "Password Changed",
                        message: `Password for ${params.username} changed successfully.`,
                    });
                });
        },
        confirmation: [
            ...convertArrayOfStringsToContent([
                "Insert the new password for the user.",
            ]),
            {
                input: {
                    type: "text",
                    label: "Password",
                    key: "password",
                    required: true,
                    autofocus: true,
                },
            },
        ],
        confirmationHeader: ({ params }) =>
            `Change password for ${params.username}`,
    },
    {
        key: "logout",
        label: "Logout",
        action: async ({ params }) => {
            const cluster = !!getInputValue<string>(KEY_MODAL, "cluster");
            if (cluster) {
                return client.value
                    ?.cluster_admin_user_logout({
                        username: params.username,
                    })
                    .then(() => {
                        addNotification({
                            type: "success",
                            title: "User Logged Out",
                            message: `User ${params.username} logged out successfully from all nodes in the cluster.`,
                        });
                    });
            }
            return client.value
                ?.admin_user_logout({
                    username: params.username,
                })
                .then(() => {
                    addNotification({
                        type: "success",
                        title: "User Logged Out",
                        message: `User ${params.username} logged out successfully.`,
                    });
                });
        },
        confirmation: [
            ...convertArrayOfStringsToContent([
                "Do you want to log out the user?",
            ]),
            {
                input: {
                    type: "checkbox",
                    label: "Logout from all nodes in the cluster",
                    key: "cluster",
                    value: false,
                },
            },
        ],
        confirmationHeader: ({ params }) => `Logout ${params.username}`,
    },
    {
        key: "delete",
        label: "Delete",
        action: async ({ params }) => {
            return client.value
                ?.admin_user_delete({ username: params.username })
                .then(() => {
                    addNotification({
                        type: "success",
                        title: "User Deleted",
                        message: `User ${params.username} deleted successfully.`,
                    });
                });
        },
        confirmation: [
            ...convertArrayOfStringsToContent([
                "Do you want to delete the user?",
            ]),
        ],
        confirmationHeader: ({ params }) => `Delete ${params.username}`,
    },
];

const userColumns: Column<TRow>[] = [
    {
        key: "username",
        title: "Username",
    },
    {
        key: "admin",
        title: "Admin",
        type: "boolean",
    },
    {
        key: "login",
        title: "Logged In",
        type: "boolean",
    },
    {
        key: "actions",
        title: "Actions",
        actions: userActions as unknown as Action<TRow>[],
    },
];

export { userActions, userColumns };
