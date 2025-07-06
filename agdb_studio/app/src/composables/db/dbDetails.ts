import type { DbUserRole, ServerDatabase } from "@agnesoft/agdb_api/openapi";
import { useDbStore } from "./dbStore";
import { useDbUsersStore } from "./dbUsersStore";
import { useContentInputs } from "@agdb-studio/common/src/composables/content/inputs";
import { computed, type Ref } from "vue";
import useModal from "@agdb-studio/common/src/composables/modal/modal";
import { KEY_MODAL } from "@agdb-studio/common/src/composables/modal/constants";
import { EMPHASIZED_CLASSNAME } from "@agdb-studio/common/src/composables/content/utils";
import type { DbIdentification } from "./types";

export type DbDetailsParams = DbIdentification & Pick<ServerDatabase, "role">;

const { getDbUsers, fetchDbUsers, removeUser, addUser, isDbRoleType } =
  useDbUsersStore();
const { getInputValue } = useContentInputs();

const { openModal } = useModal();
const { getDbName } = useDbStore();

export type AddUserParams = {
  username?: string;
  db_role?: string;
};

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

  const handleRemoveUser = (username: string): void => {
    if (!canEditUsers.value) {
      return;
    }
    openModal({
      header: "Remove user",
      content: [
        {
          paragraph: [
            { text: "Are you sure you want to remove user " },
            { text: username, className: EMPHASIZED_CLASSNAME },
            { text: " from database " },
            { text: dbName.value, className: EMPHASIZED_CLASSNAME },
            { text: "?" },
          ],
        },
      ],

      onConfirm: () =>
        removeUser({
          owner: dbParams.value.owner,
          db: dbParams.value.db,
          username: username,
        }).then(() => {
          fetchDbUsers(dbParams.value);
        }),
    });
  };

  const handleAddUser = (
    { username, db_role }: AddUserParams = {
      username: undefined,
      db_role: undefined,
    },
  ): void => {
    if (!canEditUsers.value) {
      return;
    }
    openModal({
      header: "Add user",
      content: [
        {
          paragraph: [
            { text: "Add/change user role in the database " },
            { text: dbName.value, className: EMPHASIZED_CLASSNAME },
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
            value: db_role || "write",
            required: true,
          },
        },
      ],
      onConfirm: () => {
        const username = getInputValue<string>(
          KEY_MODAL,
          "username",
        )?.toString();
        const db_role = getInputValue<string>(KEY_MODAL, "role")?.toString();

        if (username?.length && db_role && isDbRoleType(db_role)) {
          return addUser({
            ...dbParams.value,
            username,
            db_role,
          }).then(() => {
            fetchDbUsers(dbParams.value);
          });
        }
        return false;
      },
    });
  };

  const isOwner = (username: string) => {
    return username === dbParams.value.owner;
  };

  const handleUsernameClick = (username: string, role: DbUserRole) => {
    if (isOwner(username) || !canEditUsers.value) {
      return;
    }
    handleAddUser({ username, db_role: role });
  };

  return {
    users,
    dbName,
    canEditUsers,
    handleRemoveUser,
    handleAddUser,
    isOwner,
    handleUsernameClick,
  };
};
