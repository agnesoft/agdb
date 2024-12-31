import type { ServerDatabase } from "agdb_api/dist/openapi";
import { client } from "@/services/api.service";
// import type { TRow } from "../table/types";

type DbActionProps = ActionProps<ServerDatabase>;
const dbActions: Action[] = [
    {
        key: "backup",
        label: "Backup",
        action: ({ params }: DbActionProps) => client.value?.db_backup(params),
    },
    {
        key: "restore",
        label: "Restore",
        action: ({ params }: DbActionProps) => client.value?.db_restore(params),
    },
    {
        key: "clear",
        label: "Clear",
        actions: [
            {
                key: "all",
                label: "All",
                action: ({ params }: DbActionProps) =>
                    client.value?.db_clear({ ...params, resource: "all" }),
            },
            {
                key: "db",
                label: "Db only",
                action: ({ params }: DbActionProps) =>
                    client.value?.db_clear({ ...params, resource: "db" }),
            },
            {
                key: "audit",
                label: "Audit only",
                action: ({ params }: DbActionProps) =>
                    client.value?.db_clear({ ...params, resource: "audit" }),
            },
            {
                key: "backup",
                label: "Backup only",
                action: ({ params }: DbActionProps) =>
                    client.value?.db_clear({ ...params, resource: "backup" }),
            },
        ],
    },
    {
        key: "convert",
        label: "Convert",
        actions: [
            {
                key: "memory",
                label: "Memory",
                action: ({ params }: DbActionProps) =>
                    client.value?.db_convert({ ...params, db_type: "memory" }),
            },
            {
                key: "file",
                label: "File",
                action: ({ params }: DbActionProps) =>
                    client.value?.db_convert({ ...params, db_type: "file" }),
            },
            {
                key: "mapped",
                label: "Mapped",
                action: ({ params }: DbActionProps) =>
                    client.value?.db_convert({ ...params, db_type: "mapped" }),
            },
        ],
    },
    // todo: implement input for db name
    // {
    //     key: "copy",
    //     label: "Copy",
    //     action: ({ params }: DbActionProps) => client.value?.db_copy(params),
    // },
    // {
    //     key: "rename",
    //     label: "Rename",
    //     action: ({ params }: DbActionProps) => client.value?.db_rename(params),
    // }
    {
        key: "remove",
        label: "Remove",
        action: ({ params }: DbActionProps) => client.value?.db_remove(params),
    },
    {
        key: "delete",
        label: "Delete",
        action: ({ params }: DbActionProps) => client.value?.db_delete(params),
    },
    {
        key: "optimize",
        label: "Optimize",
        action: ({ params }: DbActionProps) =>
            client.value?.db_optimize(params),
    },
    {
        key: "audit",
        label: "Audit",
        action: ({ params }: DbActionProps) =>
            client.value?.db_audit(params).then((res) => {
                console.log(res.data);
            }),
    },
];

export default dbActions;
