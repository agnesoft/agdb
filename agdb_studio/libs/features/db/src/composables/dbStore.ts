import { ref } from "vue";
import type { DbKind, ServerDatabase } from "@agnesoft/agdb_api/openapi";
import { useAccount } from "@agdb-studio/auth/src/account";
import { addNotification } from "@agdb-studio/notification/src/composables/notificationStore";
import type { AxiosResponse } from "axios";
import type { DbIdentification } from "./types";
import { dbAdd, dbList } from "./dbActions";

const databases = ref<ServerDatabase[]>([]);

const fetchDatabases = async () => {
  dbList().then((dbs: AxiosResponse<ServerDatabase[]>) => {
    databases.value = dbs.data;
  });
};

export type AddDatabaseProps = {
  name: string;
  db_type: DbKind;
};

const { username } = useAccount();

const addDatabase = async ({ name, db_type }: AddDatabaseProps) => {
  if (!username.value) {
    return;
  }

  dbAdd({
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
