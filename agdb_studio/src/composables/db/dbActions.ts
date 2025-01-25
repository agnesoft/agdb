import { useAdmin } from "../user/admin";
import { checkClient, client } from "@/services/api.service";
import type {
    Components,
    DbResource,
    DbType,
    DbUser,
    DbUserRole,
    ServerDatabase,
} from "agdb_api/dist/openapi";
import type { AxiosResponse } from "axios";
import type { DbIdentification } from "./types";

const { isAdminView, isAdmin } = useAdmin();

const shouldRunAdminAction = () => isAdminView.value && isAdmin.value;

export const dbAdd = async (params: {
    owner: string;
    db: string;
    db_type: DbType;
}): Promise<AxiosResponse> => {
    checkClient(client);
    if (shouldRunAdminAction()) {
        return client.value.admin_db_add(params);
    }

    return client.value.db_add(params);
};

export const dbAudit = async (
    params: DbIdentification,
): Promise<AxiosResponse<Components.Schemas.DbAudit>> => {
    checkClient(client);
    if (shouldRunAdminAction()) {
        return client.value.admin_db_audit(params);
    }

    return client.value.db_audit(params);
};

export const dbBackup = async (
    params: DbIdentification,
): Promise<AxiosResponse> => {
    checkClient(client);
    if (shouldRunAdminAction()) {
        return client.value.admin_db_backup(params);
    }

    return client.value.db_backup(params);
};

export const dbClear = async (
    params: DbIdentification & { resource: DbResource },
): Promise<AxiosResponse> => {
    checkClient(client);
    if (shouldRunAdminAction()) {
        return client.value.admin_db_clear(params);
    }

    return client.value.db_clear(params);
};

export const dbConvert = async (
    params: DbIdentification & { db_type: DbType },
): Promise<AxiosResponse> => {
    checkClient(client);
    if (shouldRunAdminAction()) {
        return client.value.admin_db_convert(params);
    }

    return client.value.db_convert(params);
};

export const dbCopy = async (
    params: (DbIdentification & { new_db: string }) & {
        new_db: string;
        new_owner: string;
    },
): Promise<AxiosResponse> => {
    checkClient(client);
    if (shouldRunAdminAction()) {
        return client.value.admin_db_copy(params);
    }
    const { new_owner, ...rest } = params; // eslint-disable-line @typescript-eslint/no-unused-vars

    return client.value.db_copy(rest);
};

export const dbDelete = async (
    params: DbIdentification,
): Promise<AxiosResponse> => {
    checkClient(client);
    if (shouldRunAdminAction()) {
        return client.value.admin_db_delete(params);
    }

    return client.value.db_delete(params);
};

export const dbExec = async (
    params: DbIdentification & { sql: string },
): Promise<AxiosResponse> => {
    checkClient(client);
    if (shouldRunAdminAction()) {
        return client.value.admin_db_exec(params);
    }

    return client.value.db_exec(params);
};

export const dbExecMut = async (
    params: DbIdentification & { sql: string },
): Promise<AxiosResponse> => {
    checkClient(client);
    if (shouldRunAdminAction()) {
        return client.value.admin_db_exec_mut(params);
    }

    return client.value.db_exec_mut(params);
};

export const dbList = async (): Promise<AxiosResponse<ServerDatabase[]>> => {
    checkClient(client);
    if (shouldRunAdminAction()) {
        return client.value.admin_db_list();
    }

    return client.value.db_list();
};

export const dbOptimize = async (
    params: DbIdentification,
): Promise<AxiosResponse> => {
    checkClient(client);
    if (shouldRunAdminAction()) {
        return client.value.admin_db_optimize(params);
    }

    return client.value.db_optimize(params);
};

export const dbRemove = async (
    params: DbIdentification,
): Promise<AxiosResponse> => {
    checkClient(client);
    if (shouldRunAdminAction()) {
        return client.value.admin_db_remove(params);
    }

    return client.value.db_remove(params);
};

export const dbRename = async (
    params: (DbIdentification & { new_db: string }) & {
        new_db: string;
        new_owner: string;
    },
): Promise<AxiosResponse> => {
    checkClient(client);
    if (shouldRunAdminAction()) {
        return client.value.admin_db_rename(params);
    }
    const { new_owner, ...rest } = params; // eslint-disable-line @typescript-eslint/no-unused-vars

    return client.value.db_rename(rest);
};

export const dbRestore = async (
    params: DbIdentification,
): Promise<AxiosResponse> => {
    checkClient(client);
    if (shouldRunAdminAction()) {
        return client.value.admin_db_restore(params);
    }

    return client.value.db_restore(params);
};

export const dbUserAdd = async (
    params: {
        username: string;
        db_role: DbUserRole;
    } & DbIdentification,
): Promise<AxiosResponse> => {
    checkClient(client);
    if (shouldRunAdminAction()) {
        return client.value.admin_db_user_add(params);
    }

    return client.value.db_user_add(params);
};

export const dbUserList = async (
    params: DbIdentification,
): Promise<AxiosResponse<DbUser[]>> => {
    checkClient(client);
    if (shouldRunAdminAction()) {
        return client.value.admin_db_user_list(params);
    }

    return client.value.db_user_list(params);
};

export const dbUserRemove = async (
    params: {
        username: string;
    } & DbIdentification,
): Promise<AxiosResponse> => {
    checkClient(client);
    if (shouldRunAdminAction()) {
        return client.value.admin_db_user_remove(params);
    }

    return client.value.db_user_remove(params);
};
