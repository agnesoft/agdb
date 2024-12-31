import type { ServerDatabase } from "agdb_api/dist/openapi";
import { client } from "@/services/api.service";
// import type { TRow } from "../table/types";

const dbActions: Action[] = [
    {
        key: "backup",
        label: "Backup",
        action: (row: ServerDatabase) =>
            client.value?.db_backup({ db: row.db, owner: row.owner }),
    },
    {
        key: "restore",
        label: "Restore",
        action: (db: ServerDatabase) => client.value?.db_restore(db),
    },
    // clear: {
    //     label: "Clear",
    //     action: (db: ServerDatabase) =>
    //         client.value?.db_clear(db),
    // },
    {
        key: "remove",
        label: "Remove",
        action: (db: ServerDatabase) => client.value?.db_remove(db),
    },
];

export default dbActions;
