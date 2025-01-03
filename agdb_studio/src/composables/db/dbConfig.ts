import type { ServerDatabase } from "agdb_api/dist/openapi";
import { client } from "@/services/api.service";
import { dateFormatter } from "@/composables/table/utils";
import { convertArrayOfStringsToContent } from "@/composables/content/utils";
import { useContentInputs } from "../content/inputs";
import { KEY_MODAL } from "../modal/constants";
import useModal from "../modal/modal";

const { getInputValue } = useContentInputs();
const { showModal } = useModal();

export type DbActionProps = ActionProps<ServerDatabase>;

const getConfirmationHeaderFn = ({ params }: DbActionProps) =>
    `Confirm action for ${params.owner}/${params.db}`;

const dbActions: Action[] = [
    {
        key: "audit",
        label: "Audit",
        action: ({ params }: DbActionProps) =>
            client.value?.db_audit(params).then((res) => {
                const content = res.data.length
                    ? convertArrayOfStringsToContent(
                          res.data.map(
                              (item) =>
                                  `${item.timestamp} | ${item.user} | ${item.query}`,
                          ),
                      )
                    : convertArrayOfStringsToContent(["No audit logs found."]);

                showModal({
                    header: `Audit log of ${params.owner}/${params.db}`,
                    content,
                });
            }),
    },
    {
        key: "backup",
        label: "Backup",
        action: ({ params }: DbActionProps) => client.value?.db_backup(params),
        confirmation: [
            ...convertArrayOfStringsToContent(
                ["Are you sure you want to backup this database?"],
                { emphesizedWords: ["backup"] },
            ),
            ...convertArrayOfStringsToContent([
                "This will swap the existing backup with the current db.",
            ]),
        ],
        confirmationHeader: getConfirmationHeaderFn,
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
                confirmation: convertArrayOfStringsToContent(
                    [
                        "Are you sure you want to clear all resources of this database?",
                        "This will reset the database.",
                    ],
                    { emphesizedWords: ["clear", "all"] },
                ),
                confirmationHeader: getConfirmationHeaderFn,
            },
            {
                key: "db",
                label: "Db only",
                action: ({ params }: DbActionProps) =>
                    client.value?.db_clear({ ...params, resource: "db" }),
                confirmation: convertArrayOfStringsToContent(
                    [
                        "Are you sure you want to clear this database?",
                        "This will remove all data.",
                    ],
                    { emphesizedWords: ["clear", "database"] },
                ),
                confirmationHeader: getConfirmationHeaderFn,
            },
            {
                key: "audit",
                label: "Audit only",
                action: ({ params }: DbActionProps) =>
                    client.value?.db_clear({ ...params, resource: "audit" }),
                confirmation: convertArrayOfStringsToContent(
                    [
                        "Are you sure you want to clear the audit log of this database?",
                    ],
                    { emphesizedWords: ["clear", "audit"] },
                ),
                confirmationHeader: getConfirmationHeaderFn,
            },
            {
                key: "backup",
                label: "Backup only",
                action: ({ params }: DbActionProps) =>
                    client.value?.db_clear({ ...params, resource: "backup" }),
                confirmation: convertArrayOfStringsToContent(
                    [
                        "Are you sure you want to clear the backup of this database?",
                    ],
                    { emphesizedWords: ["clear", "backup"] },
                ),
                confirmationHeader: getConfirmationHeaderFn,
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
                confirmation: convertArrayOfStringsToContent(
                    [
                        "Are you sure you want to convert this database to memory only?",
                    ],
                    { emphesizedWords: ["convert", "memory"] },
                ),
                confirmationHeader: getConfirmationHeaderFn,
            },
            {
                key: "file",
                label: "File",
                action: ({ params }: DbActionProps) =>
                    client.value?.db_convert({ ...params, db_type: "file" }),
                confirmation: convertArrayOfStringsToContent(
                    [
                        "Are you sure you want to convert this database to file based database?",
                    ],
                    { emphesizedWords: ["convert", "file"] },
                ),
                confirmationHeader: getConfirmationHeaderFn,
            },
            {
                key: "mapped",
                label: "Mapped",
                action: ({ params }: DbActionProps) =>
                    client.value?.db_convert({ ...params, db_type: "mapped" }),
                confirmation: convertArrayOfStringsToContent(
                    [
                        "Are you sure you want to convert this database to memory mapped database?",
                    ],
                    { emphesizedWords: ["convert", "mapped"] },
                ),
                confirmationHeader: getConfirmationHeaderFn,
            },
        ],
    },
    {
        key: "copy",
        label: "Copy",
        action: ({ params }: DbActionProps) => {
            const new_db = getInputValue(KEY_MODAL, "new_db")?.toString();
            const { db, owner } = params;
            return new_db?.length
                ? client.value?.db_copy({ db, owner, new_db })
                : Promise.reject();
        },
        confirmation: [
            ...convertArrayOfStringsToContent([
                "Insert name of the new database.",
            ]),
            {
                input: {
                    key: "new_db",
                    label: "New name",
                    type: "text",
                    autofocus: true,
                },
            },
        ],
        confirmationHeader: getConfirmationHeaderFn,
    },
    {
        key: "delete",
        label: "Delete",
        action: ({ params }: DbActionProps) => client.value?.db_delete(params),
        confirmation: [
            ...convertArrayOfStringsToContent(
                ["Are you sure you want to delete this database?"],
                { emphesizedWords: ["delete"] },
            ),
            ...convertArrayOfStringsToContent(
                ["This will permanently delete all data."],
                { emphesizedWords: ["all data"] },
            ),
        ],
        confirmationHeader: getConfirmationHeaderFn,
    },
    {
        key: "optimize",
        label: "Optimize",
        action: ({ params }: DbActionProps) =>
            client.value?.db_optimize(params),
        confirmation: convertArrayOfStringsToContent(
            ["Are you sure you want to optimize this database?"],
            { emphesizedWords: ["optimize"] },
        ),
        confirmationHeader: getConfirmationHeaderFn,
    },

    {
        key: "remove",
        label: "Remove",
        action: ({ params }: DbActionProps) => client.value?.db_remove(params),
        confirmation: convertArrayOfStringsToContent(
            [
                "Are you sure you want to remove this database?",
                "This will only disassociate the database from the server. No data will be deleted.",
            ],
            { emphesizedWords: ["remove"] },
        ),
        confirmationHeader: getConfirmationHeaderFn,
    },
    {
        key: "rename",
        label: "Rename",
        action: ({ params }: DbActionProps) => {
            const new_db = getInputValue(KEY_MODAL, "new_db")?.toString();
            const { db, owner } = params;
            return new_db?.length
                ? client.value?.db_rename({ db, owner, new_db })
                : Promise.reject();
        },
        confirmation: [
            ...convertArrayOfStringsToContent([
                "Insert new name of the database.",
            ]),
            {
                input: {
                    key: "new_db",
                    label: "New name",
                    type: "text",
                    autofocus: true,
                },
            },
        ],
        confirmationHeader: getConfirmationHeaderFn,
    },
    {
        key: "restore",
        label: "Restore",
        action: ({ params }: DbActionProps) => client.value?.db_restore(params),
        confirmation: convertArrayOfStringsToContent(
            [
                "Are you sure you want to restore backup of this database?",
                "This will swap the existing db with the backup.",
            ],
            { emphesizedWords: ["restore"] },
        ),
        confirmationHeader: getConfirmationHeaderFn,
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

export { dbActions, dbColumns, getConfirmationHeaderFn };
