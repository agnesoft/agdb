import { client } from "@/services/api.service";
import type { DbUser, DbUserRole } from "agdb_api/dist/openapi";
import { ref } from "vue";
import { useDbStore, type DbIdentification } from "./dbStore";

const { getDbName } = useDbStore();

const dbUsers = ref(new Map<string, DbUser[]>());

const fetchDbUsers = async (params: DbIdentification) => {
    client.value?.db_user_list(params).then((users) => {
        dbUsers.value.set(getDbName(params), users.data);
    });
};

const getDbUsers = (params: DbIdentification) => {
    return dbUsers.value.get(getDbName(params));
};

const clearDbUsers = (params: DbIdentification) => {
    dbUsers.value.delete(getDbName(params));
};

const clearAllDbUsers = () => {
    dbUsers.value.clear();
};

type AddUserProps = {
    username: string;
    db_role: DbUserRole;
} & DbIdentification;
const addUser = async (params: AddUserProps) => {
    client.value?.db_user_add(params);
};

type RemoveUserProps = {
    username: string;
} & DbIdentification;
const removeUser = async (params: RemoveUserProps) => {
    client.value?.db_user_remove(params);
};

const isDbRoleType = (role: string): role is DbUserRole => {
    return ["read", "write", "admin"].includes(role);
};

export const useDbUsers = () => {
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
