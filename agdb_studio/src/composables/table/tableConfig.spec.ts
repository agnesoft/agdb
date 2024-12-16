import { getTable } from "./tableConfig";

describe("tableConfig", () => {
    it("should return table config", () => {
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
