import { addTable, getTable } from "./tableConfig";
import { setTableData } from "./tableData";

describe("tableFilter", () => {
    const TABLE_NAME = "my_table";
    const tableConfig = [
        { key: "role", title: "Role" },
        { key: "name", title: "Name" },
        { key: "db_type", title: "Type" },
        { key: "size", title: "Size" },
    ];
    const tableData = [
        { role: "admin", name: "John", db_type: "mysql", size: 10 },
        { role: "user", name: "Jane", db_type: "postgres", size: 20 },
        { role: "admin", name: "Alice", db_type: "mysql", size: 30 },
    ];
    addTable(TABLE_NAME, tableConfig);
    setTableData(TABLE_NAME, tableData);

    describe("getTableFilter", () => {
        it("should return table filter", () => {
            const table = getTable("table");
            expect(table).toEqual({
                columns: [
                    { key: "name", label: "Name" },
                    { key: "age", label: "Age" },
                    { key: "job", label: "Job" },
                ],
                data: new Map(),
            });
        });
    });
});
