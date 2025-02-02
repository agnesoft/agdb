import type { DbUser, DbUserRole } from "@agnesoft/agdb_api/dist/openapi";
import { ref } from "vue";
import { useDbStore } from "./dbStore";
import { addNotification } from "../notification/notificationStore";
import type { DbIdentification } from "./types";
import { dbUserAdd, dbUserList, dbUserRemove } from "./dbActions";

const { getDbName } = useDbStore();

const dbUsers = ref(new Map<string, DbUser[]>());

const fetchDbUsers = (params: DbIdentification): Promise<void> | undefined =>
    dbUserList(params).then((users) => {
        dbUsers.value.set(getDbName(params), users.data);
    });

const getDbUsers = (params: DbIdentification): DbUser[] | undefined => {
    return dbUsers.value.get(getDbName(params));
};

const clearDbUsers = (params: DbIdentification): void => {
    dbUsers.value.delete(getDbName(params));
};

const clearAllDbUsers = (): void => {
    dbUsers.value.clear();
};

type AddUserProps = {
    username: string;
    db_role: DbUserRole;
} & DbIdentification;
const addUser = async (params: AddUserProps) => {
    return dbUserAdd(params).then(() => {
        addNotification({
            type: "success",
            title: "User added/changed",
            message: `User ${params.username} added/change in the database ${getDbName(params)} successfully.`,
        });
    });
};

type RemoveUserProps = {
    username: string;
} & DbIdentification;
const removeUser = async (params: RemoveUserProps) => {
    return dbUserRemove(params).then(() => {
        addNotification({
            type: "success",
            title: "User removed",
            message: `User ${params.username} removed from database ${getDbName(params)} successfully.`,
        });
    });
};

const isDbRoleType = (role: string): role is DbUserRole => {
    return ["read", "write", "admin"].includes(role);
};

export const useDbUsersStore = () => {
    return {
        getDbUsers,
        fetchDbUsers,
        addUser,
        removeUser,
        clearDbUsers,
        clearAllDbUsers,
        isDbRoleType,
    };
};
