export const TABLE_NAME = "my_table";
export const tableConfig = [
    { key: "role", title: "Role" },
    { key: "name", title: "Name" },
    { key: "db_type", title: "Type" },
    { key: "size", title: "Size" },
    {
        key: "backup",
        title: "Backup",
        valueFormatter: <T>(value: T) => (value ? "1" : "0"),
    },
];

export const columnsMap = new Map();
tableConfig.forEach((column) => {
    columnsMap.set(column.key, column);
});
export const tableData = [
    {
        role: "admin",
        name: "admin/app1",
        db_type: "memory",
        size: 10,
        backup: 0,
    },
    { role: "user", name: "user/app1", db_type: "file", size: 20, backup: 0 },
    { role: "admin", name: "admin/app3", db_type: "file", size: 50, backup: 0 },
    {
        role: "admin",
        name: "admin/app2",
        db_type: "mapped",
        size: 20,
        backup: 0,
    },
    { role: "user", name: "user/app2", db_type: "memory", size: 40, backup: 0 },
];
