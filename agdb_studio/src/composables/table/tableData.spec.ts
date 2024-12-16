import { getTable } from "./tableConfig";

describe("tableData", () => {
    it("should return table data", () => {
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
