import { ref } from "vue";
import { client } from "@/services/api.service";
import type { DbType, ServerDatabase } from "agdb_api/dist/openapi";
import { useAccount } from "../user/account";
import { addNotification } from "../notification/notificationStore";
import type { AxiosResponse } from "axios";
import { useAdmin } from "../user/admin";
import type { DbIdentification } from "./types";

const { isAdminView } = useAdmin();

enum DbAction {
    DB_ADD = "db_add",
    DB_AUDIT = "db_audit",
    DB_BACKUP = "db_backup",
    DB_CLEAR = "db_clear",
    DB_CONVERT = "db_convert",
    DB_COPY = "db_copy",
    DB_DELETE = "db_delete",
    DB_EXEC = "db_exec",
    DB_EXEC_MUT = "db_exec_mut",
    DB_LIST = "db_list",
    DB_OPTIMIZE = "db_optimize",
    DB_REMOVE = "db_remove",
    DB_RENAME = "db_rename",
    DB_RESTORE = "db_restore",
    DB_USER_ADD = "db_user_add",
    DB_USER_LIST = "db_user_list",
    DB_USER_REMOVE = "db_user_remove",
}

enum AdminDbAction {
    ADMIN_DB_ADD = "admin_db_add",
    ADMIN_DB_AUDIT = "admin_db_audit",
    ADMIN_DB_BACKUP = "admin_db_backup",
    ADMIN_DB_CLEAR = "admin_db_clear",
    ADMIN_DB_CONVERT = "admin_db_convert",
    ADMIN_DB_COPY = "admin_db_copy",
    ADMIN_DB_DELETE = "admin_db_delete",
    ADMIN_DB_EXEC = "admin_db_exec",
    ADMIN_DB_EXEC_MUT = "admin_db_exec_mut",
    ADMIN_DB_LIST = "admin_db_list",
    ADMIN_DB_OPTIMIZE = "admin_db_optimize",
    ADMIN_DB_REMOVE = "admin_db_remove",
    ADMIN_DB_RENAME = "admin_db_rename",
    ADMIN_DB_RESTORE = "admin_db_restore",
    ADMIN_DB_USER_ADD = "admin_db_user_add",
    ADMIN_DB_USER_LIST = "admin_db_user_list",
    ADMIN_DB_USER_REMOVE = "admin_db_user_remove",
}

const mapDbAction = (action: DbAction): AdminDbAction => {
    return ("admin_" + action) as AdminDbAction;
};

const runDbAction = <T, R>(
    name: DbAction,
): ((params?: T) => Promise<AxiosResponse<R>>) | undefined => {
    if (!isAdminView.value) {
        return client.value?.[mapDbAction(name)] as
            | ((params?: T) => Promise<AxiosResponse<R>>)
            | undefined;
    }
    return client.value?.[name] as
        | ((params?: T) => Promise<AxiosResponse<R>>)
        | undefined;
};

// const runDbAction = <T>(
//     name: DbAction,
// ): ((params?: T) => Promise<AxiosResponse<any, any>>) | undefined => {
//     if (!isAdminView.value) {
//         return client.value?.[mapDbAction(name)] as
//             | ((params?: T) => Promise<AxiosResponse<any, any>>)
//             | undefined;
//     }
//     return client.value?.[name] as
//         | ((params?: T) => Promise<AxiosResponse<any, any>>)
//         | undefined;
// };

const databases = ref<ServerDatabase[]>([]);

const fetchDatabases = async () => {
    runDbAction<void, ServerDatabase[]>(DbAction.DB_LIST)?.().then(
        (dbs: AxiosResponse<ServerDatabase[]>) => {
            databases.value = dbs.data;
        },
    );
};

export type AddDatabaseProps = {
    name: string;
    db_type: DbType;
};

const { username } = useAccount();

const addDatabase = async ({ name, db_type }: AddDatabaseProps) => {
    if (!username.value) {
        return;
    }

    client.value?.["db_add"]({
        owner: username.value,
        db: name,
        db_type,
    }).then(() => {
        addNotification({
            type: "success",
            title: "Database added",
            message: `Database ${name} added successfully.`,
        });
    });

    // runDbAction<{ owner: string; db: string; db_type: DbType }, any>(
    //     DbAction.DB_ADD,
    // )?.({
    //     owner: username.value,
    //     db: name,
    //     db_type,
    // }).then(() => {
    //     addNotification({
    //         type: "success",
    //         title: "Database added",
    //         message: `Database ${name} added successfully.`,
    //     });
    // });
};

const getDbName = (db: DbIdentification) => {
    return `${db.owner}/${db.db}`;
};

export const useDbStore = () => {
    return {
        databases,
        fetchDatabases,
        addDatabase,
        getDbName,
    };
};
