import { addTable, getTable } from "./tableConfig";
import {
    addRow,
    removeRow,
    setTableData,
    clearTableData,
    getRows,
} from "./tableData";
import { TABLE_NAME, tableConfig, tableData } from "@/tests/tableMocks";
import { addFilter, getTableFilter, setSort } from "./tableFilter";

describe("tableData", () => {
    addTable({ name: TABLE_NAME, columns: tableConfig, uniqueKey: "name" });

    beforeEach(() => {
        const table = getTable(TABLE_NAME);
        table?.data?.clear();
    });

    describe("setTableData", () => {
        it("should set table data", () => {
            const table = getTable(TABLE_NAME);
            expect(table?.data?.size).toBe(0);
            setTableData(TABLE_NAME, tableData);
            expect(table?.data?.size).toBe(5);
        });
        it("should not set table data if table does not exist", () => {
            const table = getTable("non_existent_table");
            expect(table).toBeUndefined();
            setTableData("non_existent_table", tableData);
            expect(table).toBeUndefined();
        });
        it("should set table data with default keys if unique key doesn't exist", () => {
            addTable({
                name: "table_without_unique_key",
                columns: tableConfig,
            });
            setTableData("table_without_unique_key", tableData);
            const table = getTable("table_without_unique_key");
            expect([...(table?.data?.keys() ?? [])]).toStrictEqual([
                "0",
                "1",
                "2",
                "3",
                "4",
            ]);
        });
    });

    describe("addRow", () => {
        it("should add rows", () => {
            const table = getTable(TABLE_NAME);
            expect(table?.data?.size).toBe(0);
            tableData.forEach((row) => {
                addRow(TABLE_NAME, row);
            });
            expect(table?.data?.size).toBe(5);
        });
        it("should not add row if table does not exist", () => {
            const table = getTable("non_existent_table");
            expect(table).toBeUndefined();
            addRow("non_existent_table", tableData[0]);
            expect(table).toBeUndefined();
        });
    });

    describe("removeRow", () => {
        it("should remove row", () => {
            const table = getTable(TABLE_NAME);
            setTableData(TABLE_NAME, tableData);
            expect(table?.data?.get("admin/app1")).toBeDefined();
            expect(table?.data?.size).toBe(5);
            removeRow(TABLE_NAME, "admin/app1");
            expect(table?.data?.size).toBe(4);
            expect(table?.data?.get("admin/app1")).toBeUndefined();
        });
    });

    describe("clearTableData", () => {
        it("should clear table data", () => {
            const table = getTable(TABLE_NAME);
            setTableData(TABLE_NAME, tableData);
            expect(table?.data?.size).toBe(5);
            clearTableData(TABLE_NAME);
            expect(table?.data?.size).toBe(0);
        });
    });

    describe("getRows", () => {
        beforeEach(() => {
            const filter = getTableFilter(TABLE_NAME);
            filter?.filters.clear();
            filter?.sort.clear();
        });
        it("should return all rows when filters are not set", () => {
            setTableData(TABLE_NAME, tableData);
            expect(getRows(TABLE_NAME).length).toBe(5);
        });

        it("should return filtered rows", () => {
            setTableData(TABLE_NAME, tableData);
            addFilter(TABLE_NAME, "role", "admin");
            expect(getRows(TABLE_NAME).length).toBe(3);
        });

        it("should return filtered rows with multiple filters", () => {
            setTableData(TABLE_NAME, tableData);
            addFilter(TABLE_NAME, "role", "admin");
            addFilter(TABLE_NAME, "db_type", "memory");
            expect(getRows(TABLE_NAME).length).toBe(1);
        });

        it("should return empty array if no rows match filter", () => {
            setTableData(TABLE_NAME, tableData);
            addFilter(TABLE_NAME, "role", "non_existent_role");
            expect(getRows(TABLE_NAME).length).toBe(0);
        });

        it("should return sorted rows in desc order", () => {
            setTableData(TABLE_NAME, tableData);
            setSort(TABLE_NAME, "size", "desc");
            expect(getRows(TABLE_NAME)[0][1].name).toBe("admin/app3");
        });
        it("should return sorted rows in asc order", () => {
            setTableData(TABLE_NAME, tableData);
            setSort(TABLE_NAME, "size", "asc");
            expect(getRows(TABLE_NAME)[0][1].name).toBe("admin/app1");
        });
        it("should return sorted rows with multiple sort keys", () => {
            setTableData(TABLE_NAME, tableData);
            setSort(TABLE_NAME, "role", "asc");
            setSort(TABLE_NAME, "name", "desc");
            expect(getRows(TABLE_NAME)[0][1].name).toBe("user/app2");
        });
        it("should return empty array if table doesn't exist", () => {
            expect(getRows("non_existent_table").length).toBe(0);
        });
    });
});
