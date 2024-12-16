import { addTable, getTable } from "./tableConfig";
import { TABLE_NAME, tableConfig, columnsMap } from "@/tests/tableMocks";

describe("tableData", () => {
    addTable({ name: TABLE_NAME, columns: tableConfig, uniqueKey: "name" });

    it("should return table config", () => {
        const table = getTable(TABLE_NAME);
        expect(table).toEqual({
            name: TABLE_NAME,
            columns: columnsMap,
            data: new Map(),
            uniqueKey: "name",
        });
    });
});
