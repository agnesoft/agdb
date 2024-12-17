import {
    addTable,
    clearTables,
    getTable,
    getTableColumns,
    getTableColumnsArray,
    removeTable,
    tableExists,
} from "./tableConfig";
import { TABLE_NAME, tableConfig, columnsMap } from "@/tests/tableMocks";

describe("tableData", () => {
    beforeEach(() => {
        clearTables();
    });
    it("should return table configs", () => {
        addTable({ name: TABLE_NAME, columns: tableConfig, uniqueKey: "name" });
        const table = getTable(TABLE_NAME);
        expect(table).toEqual({
            name: TABLE_NAME,
            columns: columnsMap,
            data: new Map(),
            uniqueKey: "name",
        });
        const columns = getTableColumns(TABLE_NAME);
        expect(columns).toEqual(columnsMap);

        const columnsArray = getTableColumnsArray(TABLE_NAME);
        expect(columnsArray).toEqual(tableConfig);
    });

    it("should return undefined if table does not exist", () => {
        const table = getTable("non_existent_table");
        expect(table).toBeUndefined();

        const columns = getTableColumns("non_existent_table");
        expect(columns).toEqual(new Map());

        const columnsArray = getTableColumnsArray("non_existent_table");
        expect(columnsArray).toEqual([]);
    });

    it("should remove table", () => {
        addTable({ name: TABLE_NAME, columns: tableConfig, uniqueKey: "name" });
        const table = getTable(TABLE_NAME);
        expect(table).toBeDefined();
        removeTable(TABLE_NAME);
        const clearedTable = getTable(TABLE_NAME);
        expect(clearedTable).toBeUndefined();
    });

    it("should check if table exists", () => {
        addTable({ name: TABLE_NAME, columns: tableConfig, uniqueKey: "name" });
        expect(tableExists(TABLE_NAME)).toBeTruthy();
        expect(tableExists("non_existent_table")).toBeFalsy();
    });
});
