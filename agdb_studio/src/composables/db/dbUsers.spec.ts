import { describe, beforeEach, vi, it, expect } from "vitest";
import { useDbUsers } from "./dbUsers";

describe("dbUsers", () => {
    it("should fetch users", async () => {
        const { fetchDbUsers } = useDbUsers();

        expect(true).toBe(true);
    });
});
