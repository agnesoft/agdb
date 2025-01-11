import { ref } from "vue";
import { client } from "@/services/api.service";
import type { DbType, ServerDatabase } from "agdb_api/dist/openapi";
import { useAccount } from "../user/account";
import { useNotificationStore } from "../notification/notificationStore";

const databases = ref<ServerDatabase[]>([]);

const fetchDatabases = async () => {
    client.value?.db_list().then((dbs) => {
        databases.value = dbs.data;
    });
};

export type AddDatabaseProps = {
    name: string;
    db_type: DbType;
};

const { username } = useAccount();

const { addNotification } = useNotificationStore();

const addDatabase = async ({ name, db_type }: AddDatabaseProps) => {
    if (!username.value) {
        return;
    }
    client.value
        ?.db_add({ owner: username.value, db: name, db_type })
        .then(() => {
            addNotification({
                type: "success",
                title: "Database added",
                message: `Database ${name} added successfully`,
            });
        });
};

export type DbIdentification = Pick<ServerDatabase, "owner" | "db">;

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
