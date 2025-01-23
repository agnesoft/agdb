import type { ServerDatabase } from "agdb_api/dist/openapi";
import { client } from "@/services/api.service";
import { dateFormatter } from "@/composables/table/utils";
import { convertArrayOfStringsToContent } from "@/composables/content/utils";
import { useContentInputs } from "../content/inputs";
import { KEY_MODAL } from "../modal/constants";
import useModal from "../modal/modal";
import { addNotification } from "../notification/notificationStore";
import { useDbStore } from "./dbStore";
import { dbCopy } from "./dbActions";

const { getInputValue } = useContentInputs();
const { openModal } = useModal();
const { getDbName } = useDbStore();

export type DbActionProps = ActionProps<ServerDatabase>;

const getConfirmationHeaderFn: ActionFn<ServerDatabase, string> = ({
    params,
}: DbActionProps) => `Confirm action for ${params.owner}/${params.db}`;

const dbActions: Action<ServerDatabase>[] = [
    {
        key: "audit",
        label: "Audit",
        action: ({ params }: DbActionProps) =>
            client.value?.db_audit(params).then((res) => {
                const content = res.data.length
                    ? convertArrayOfStringsToContent(
                          res.data.map(
                              (item) =>
                                  `${item.timestamp} | ${item.username} | ${item.query}`,
                          ),
                      )
                    : convertArrayOfStringsToContent(["No audit logs found."]);

                openModal({
                    header: `Audit log of ${params.owner}/${params.db}`,
                    content,
                });
            }),
    },
    {
        key: "backup",
        label: "Backup",
        action: ({ params }: DbActionProps) =>
            client.value?.db_backup(params).then(() => {
                addNotification({
                    type: "success",
                    title: "Backup created",
                    message: `Backup of ${getDbName(params)} has been created successfully.`,
                });
            }),
        confirmation: [
            ...convertArrayOfStringsToContent(
                ["Are you sure you want to backup this database?"],
                { emphasizedWords: ["backup"] },
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
                    client.value
                        ?.db_clear({ ...params, resource: "all" })
                        .then(() => {
                            addNotification({
                                type: "success",
                                title: "Database cleared",
                                message: `Database ${getDbName(
                                    params,
                                )} has been cleared successfully.`,
                            });
                        }),
                confirmation: convertArrayOfStringsToContent(
                    [
                        "Are you sure you want to clear all resources of this database?",
                        "This will reset the database.",
                    ],
                    { emphasizedWords: ["clear", "all"] },
                ),
                confirmationHeader: getConfirmationHeaderFn,
            },
            {
                key: "db",
                label: "Db only",
                action: ({ params }: DbActionProps) =>
                    client.value
                        ?.db_clear({ ...params, resource: "db" })
                        .then(() => {
                            addNotification({
                                type: "success",
                                title: "Database cleared",
                                message: `The data of ${getDbName(
                                    params,
                                )} has been cleared successfully.`,
                            });
                        }),
                confirmation: convertArrayOfStringsToContent(
                    [
                        "Are you sure you want to clear this database?",
                        "This will remove all data.",
                    ],
                    { emphasizedWords: ["clear", "database"] },
                ),
                confirmationHeader: getConfirmationHeaderFn,
            },
            {
                key: "audit",
                label: "Audit only",
                action: ({ params }: DbActionProps) =>
                    client.value
                        ?.db_clear({ ...params, resource: "audit" })
                        .then(() => {
                            addNotification({
                                type: "success",
                                title: "Audit log cleared",
                                message: `Audit log of ${getDbName(
                                    params,
                                )} has been cleared successfully.`,
                            });
                        }),
                confirmation: convertArrayOfStringsToContent(
                    [
                        "Are you sure you want to clear the audit log of this database?",
                    ],
                    { emphasizedWords: ["clear", "audit"] },
                ),
                confirmationHeader: getConfirmationHeaderFn,
            },
            {
                key: "backup",
                label: "Backup only",
                action: ({ params }: DbActionProps) =>
                    client.value
                        ?.db_clear({ ...params, resource: "backup" })
                        .then(() => {
                            addNotification({
                                type: "success",
                                title: "Backup cleared",
                                message: `Backup of ${getDbName(
                                    params,
                                )} has been cleared successfully.`,
                            });
                        }),
                confirmation: convertArrayOfStringsToContent(
                    [
                        "Are you sure you want to clear the backup of this database?",
                    ],
                    { emphasizedWords: ["clear", "backup"] },
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
                    client.value
                        ?.db_convert({ ...params, db_type: "memory" })
                        .then(() => {
                            addNotification({
                                type: "success",
                                title: "Database converted",
                                message: `Database ${getDbName(params)} has been converted to memory only successfully.`,
                            });
                        }),
                confirmation: convertArrayOfStringsToContent(
                    [
                        "Are you sure you want to convert this database to memory only?",
                    ],
                    { emphasizedWords: ["convert", "memory"] },
                ),
                confirmationHeader: getConfirmationHeaderFn,
            },
            {
                key: "file",
                label: "File",
                action: ({ params }: DbActionProps) =>
                    client.value
                        ?.db_convert({ ...params, db_type: "file" })
                        .then(() => {
                            addNotification({
                                type: "success",
                                title: "Database converted",
                                message: `Database ${getDbName(params)} has been converted to file based successfully.`,
                            });
                        }),
                confirmation: convertArrayOfStringsToContent(
                    [
                        "Are you sure you want to convert this database to file based database?",
                    ],
                    { emphasizedWords: ["convert", "file"] },
                ),
                confirmationHeader: getConfirmationHeaderFn,
            },
            {
                key: "mapped",
                label: "Mapped",
                action: ({ params }: DbActionProps) =>
                    client.value
                        ?.db_convert({ ...params, db_type: "mapped" })
                        .then(() => {
                            addNotification({
                                type: "success",
                                title: "Database converted",
                                message: `Database ${getDbName(params)} has been converted to memory mapped successfully.`,
                            });
                        }),
                confirmation: convertArrayOfStringsToContent(
                    [
                        "Are you sure you want to convert this database to memory mapped database?",
                    ],
                    { emphasizedWords: ["convert", "mapped"] },
                ),
                confirmationHeader: getConfirmationHeaderFn,
            },
        ],
    },
    {
        key: "copy",
        label: "Copy",
        action: async ({ params }: DbActionProps) => {
            const new_db = getInputValue<string>(
                KEY_MODAL,
                "new_db",
            )?.toString();
            const new_owner = getInputValue<string>(
                KEY_MODAL,
                "new_db",
            )?.toString();
            const { db, owner } = params;
            return dbCopy({ db, owner, new_db, new_owner }).then(() => {
                addNotification({
                    type: "success",
                    title: "Database copied",
                    message: `Database ${getDbName(
                        params,
                    )} has been copied successfully.`,
                });
            });
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
                    required: true,
                },
            },
        ],
        confirmationHeader: getConfirmationHeaderFn,
    },
    {
        key: "delete",
        label: "Delete",
        action: ({ params }: DbActionProps) =>
            client.value?.db_delete(params).then(() => {
                addNotification({
                    type: "success",
                    title: "Database deleted",
                    message: `Database ${getDbName(params)} has been deleted successfully.`,
                });
            }),
        confirmation: [
            ...convertArrayOfStringsToContent(
                ["Are you sure you want to delete this database?"],
                { emphasizedWords: ["delete"] },
            ),
            ...convertArrayOfStringsToContent(
                ["This will permanently delete all data."],
                { emphasizedWords: ["all data"] },
            ),
        ],
        confirmationHeader: getConfirmationHeaderFn,
    },
    {
        key: "optimize",
        label: "Optimize",
        action: ({ params }: DbActionProps) =>
            client.value?.db_optimize(params).then(() => {
                addNotification({
                    type: "success",
                    title: "Database optimized",
                    message: `Database ${getDbName(params)} has been optimized successfully.`,
                });
            }),
        confirmation: convertArrayOfStringsToContent(
            ["Are you sure you want to optimize this database?"],
            { emphasizedWords: ["optimize"] },
        ),
        confirmationHeader: getConfirmationHeaderFn,
    },

    {
        key: "remove",
        label: "Remove",
        action: ({ params }: DbActionProps) =>
            client.value?.db_remove(params).then(() => {
                addNotification({
                    type: "success",
                    title: "Database removed",
                    message: `Database ${getDbName(params)} has been removed successfully.`,
                });
            }),
        confirmation: convertArrayOfStringsToContent(
            [
                "Are you sure you want to remove this database?",
                "This will only disassociate the database from the server. No data will be deleted.",
            ],
            { emphasizedWords: ["remove"] },
        ),
        confirmationHeader: getConfirmationHeaderFn,
    },
    {
        key: "rename",
        label: "Rename",
        action: ({ params }: DbActionProps) => {
            const new_db = getInputValue<string>(
                KEY_MODAL,
                "new_db",
            )?.toString();
            const { db, owner } = params;
            return client.value?.db_rename({ db, owner, new_db }).then(() => {
                addNotification({
                    type: "success",
                    title: "Database renamed",
                    message: `Database ${getDbName(
                        params,
                    )} has been renamed successfully.`,
                });
            });
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
                    required: true,
                },
            },
        ],
        confirmationHeader: getConfirmationHeaderFn,
    },
    {
        key: "restore",
        label: "Restore",
        action: ({ params }: DbActionProps) =>
            client.value?.db_restore(params).then(() => {
                addNotification({
                    type: "success",
                    title: "Database restored",
                    message: `Backup of ${getDbName(params)} has been restored successfully.`,
                });
            }),
        confirmation: convertArrayOfStringsToContent(
            [
                "Are you sure you want to restore backup of this database?",
                "This will swap the existing db with the backup.",
            ],
            { emphasizedWords: ["restore"] },
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
