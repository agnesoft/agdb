import { ref } from "vue";
import { client } from "@/services/api.service";
import type { DbType, ServerDatabase } from "agdb_api/dist/openapi";
import { useAccount } from "../user/account";

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

const addDatabase = async ({ name, db_type }: AddDatabaseProps) => {
    if (!username.value) {
        return;
    }
    client.value?.db_add({ owner: username.value, db: name, db_type });
};

export const useDbStore = () => {
    return {
        databases,
        fetchDatabases,
        addDatabase,
    };
};
