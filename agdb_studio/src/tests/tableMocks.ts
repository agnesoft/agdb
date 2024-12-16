export const TABLE_NAME = "my_table";
export const tableConfig = [
    { key: "role", title: "Role" },
    { key: "name", title: "Name" },
    { key: "db_type", title: "Type" },
    { key: "size", title: "Size" },
];

export const columnsMap = new Map();
tableConfig.forEach((column) => {
    columnsMap.set(column.key, column);
});
export const tableData = [
    { role: "admin", name: "admin/app1", db_type: "memory", size: 10 },
    { role: "user", name: "user/app1", db_type: "file", size: 20 },
    { role: "admin", name: "admin/app3", db_type: "file", size: 50 },
    { role: "admin", name: "admin/app2", db_type: "mapped", size: 30 },
    { role: "user", name: "user/app2", db_type: "memory", size: 40 },
];
