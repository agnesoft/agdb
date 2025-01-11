import type { ServerDatabase } from "agdb_api/dist/openapi";
import { useDbStore, type DbIdentification } from "./dbStore";
import { useDbUsersStore } from "./dbUsersStore";
import { useContentInputs } from "../content/inputs";
import { computed, type Ref } from "vue";
import useModal from "../modal/modal";
import { KEY_MODAL } from "../modal/constants";
import { EMPHESIZED_CLASSNAME } from "../content/utils";

export type DbDetailsParams = DbIdentification & Pick<ServerDatabase, "role">;

const { getDbUsers, fetchDbUsers, removeUser, addUser, isDbRoleType } =
    useDbUsersStore();
const { getInputValue } = useContentInputs();

const { openModal } = useModal();
const { getDbName } = useDbStore();

export const useDbDetails = (dbParams: Ref<DbDetailsParams>) => {
    const users = computed(() => {
        return getDbUsers(dbParams.value);
    });

    const dbName = computed(() => {
        return getDbName(dbParams.value);
    });

    const canEditUsers = computed(() => {
        return dbParams.value.role === "admin";
    });

    const handleRemoveUser = (username: string) => {
        if (!canEditUsers.value) {
            return;
        }
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
        if (!canEditUsers.value) {
            return;
        }
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
                const username = getInputValue(
                    KEY_MODAL,
                    "username",
                )?.toString();
                const db_role = getInputValue(KEY_MODAL, "role")?.toString();

                if (username?.length && db_role && isDbRoleType(db_role)) {
                    addUser({ ...dbParams.value, username, db_role }).then(
                        () => {
                            fetchDbUsers(dbParams.value);
                        },
                    );
                }
            },
        });
    };

    return {
        users,
        dbName,
        canEditUsers,
        handleRemoveUser,
        handleAddUser,
    };
};
