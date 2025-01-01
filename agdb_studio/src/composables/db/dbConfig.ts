import type { ServerDatabase } from "agdb_api/dist/openapi";
import { client } from "@/services/api.service";
import { dateFormatter } from "@/composables/table/utils";

type DbActionProps = ActionProps<ServerDatabase>;
const dbActions: Action[] = [
    {
        key: "audit",
        label: "Audit",
        action: ({ params }: DbActionProps) =>
            client.value?.db_audit(params).then((res) => {
                console.log(res.data);
            }),
    },
    {
        key: "backup",
        label: "Backup",
        action: ({ params }: DbActionProps) => client.value?.db_backup(params),
        confirmation: [
            "Are you sure you want to backup this database?",
            "This will swap the existing backup with the current db.",
        ],
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
                confirmation: [
                    "Are you sure you want to clear all resources of this database?",
                    "This will reset the database.",
                ],
            },
            {
                key: "db",
                label: "Db only",
                action: ({ params }: DbActionProps) =>
                    client.value?.db_clear({ ...params, resource: "db" }),
                confirmation: [
                    "Are you sure you want to clear this database?",
                    "This will remove all data.",
                ],
            },
            {
                key: "audit",
                label: "Audit only",
                action: ({ params }: DbActionProps) =>
                    client.value?.db_clear({ ...params, resource: "audit" }),
                confirmation: [
                    "Are you sure you want to clear the audit log of this database?",
                ],
            },
            {
                key: "backup",
                label: "Backup only",
                action: ({ params }: DbActionProps) =>
                    client.value?.db_clear({ ...params, resource: "backup" }),
                confirmation: [
                    "Are you sure you want to clear the backup of this database?",
                ],
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
                confirmation: [
                    "Are you sure you want to convert this database to memory only?",
                ],
            },
            {
                key: "file",
                label: "File",
                action: ({ params }: DbActionProps) =>
                    client.value?.db_convert({ ...params, db_type: "file" }),
                confirmation: [
                    "Are you sure you want to convert this database to file based database?",
                ],
            },
            {
                key: "mapped",
                label: "Mapped",
                action: ({ params }: DbActionProps) =>
                    client.value?.db_convert({ ...params, db_type: "mapped" }),
                confirmation: [
                    "Are you sure you want to convert this database to memory mapped database?",
                ],
            },
        ],
    },
    // todo: implement input for db name
    // {
    //     key: "copy",
    //     label: "Copy",
    //     action: ({ params }: DbActionProps) => client.value?.db_copy(params),
    // },
    {
        key: "delete",
        label: "Delete",
        action: ({ params }: DbActionProps) => client.value?.db_delete(params),
        confirmation: [
            "Are you sure you want to delete this database?",
            "This will permanently delete all data.",
        ],
    },
    {
        key: "optimize",
        label: "Optimize",
        action: ({ params }: DbActionProps) =>
            client.value?.db_optimize(params),
        confirmation: ["Are you sure you want to optimize this database?"],
    },

    {
        key: "remove",
        label: "Remove",
        action: ({ params }: DbActionProps) => client.value?.db_remove(params),
        confirmation: [
            "Are you sure you want to remove this database?",
            "This will only disassociate the database from the server. No data will be deleted.",
        ],
    },
    // todo: implement input for db name
    // {
    //     key: "rename",
    //     label: "Rename",
    //     action: ({ params }: DbActionProps) => client.value?.db_rename(params),
    // }
    {
        key: "restore",
        label: "Restore",
        action: ({ params }: DbActionProps) => client.value?.db_restore(params),
        confirmation: [
            "Are you sure you want to restore backup of this database?",
            "This will swap the existing db with the backup.",
        ],
    },
];

const dbColumns = [
    { key: "role", title: "Role" },
    { key: "owner", title: "Owner" },
    { key: "db", title: "Name" },
    { key: "db_type", title: "Type" },
    { key: "size", title: "Size" },
    {
        key: "backup",
        title: "Backup",
        valueFormatter: dateFormatter,
    },
    {
        key: "actions",
        title: "Actions",
        actions: dbActions,
    },
];

export { dbActions, dbColumns };
