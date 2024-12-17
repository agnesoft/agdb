import { dateFormatter } from "./utils";

describe("utils", () => {
    describe("dateFormatter", () => {
        it("should return a formatted date when value is number", () => {
            const formattedDate = dateFormatter(1734447876);
            expect(formattedDate).toBe("17. 12. 2024 16:04:36");
        });
        it("should return N/A when value is not a number", () => {
            const formattedDate = dateFormatter("not a number");
            expect(formattedDate).toBe("N/A");
        });
        it("should return N/A when value is 0", () => {
            const formattedDate = dateFormatter(0);
            expect(formattedDate).toBe("N/A");
        });
    });
});
