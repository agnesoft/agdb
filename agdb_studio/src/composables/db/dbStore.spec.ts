import { useDbStore } from "./dbStore";
import { db_list, db_add } from "@/tests/apiMock";
import { describe, beforeEach, vi, it, expect } from "vitest";

db_list.mockResolvedValue({
    data: [
        {
            name: "test_db",
            db_type: "memory",
            role: "admin",
            size: 2656,
            backup: 0,
        },
        {
            name: "test_db2",
            db_type: "memory",
            role: "admin",
            size: 2656,
            backup: 0,
        },
    ],
});

db_add.mockResolvedValue({
    data: {
        name: "test_db",
        db_type: "memory",
        role: "admin",
        size: 2656,
        backup: 0,
    },
});

const { username, admin } = vi.hoisted(() => {
    return {
        username: { value: "test_user" },
        admin: { value: false },
    };
});
vi.mock("../user/account", () => {
    return {
        useAccount: () => {
            return {
                username,
                admin,
            };
        },
    };
});

describe("DbStore", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("fetches databases when called", async () => {
        const { databases, fetchDatabases } = useDbStore();
        await fetchDatabases();
        expect(databases.value).toHaveLength(2);
    });

    it("adds a database when called", async () => {
        expect(db_add).not.toHaveBeenCalled();
        const { addDatabase } = useDbStore();
        await addDatabase({ name: "test_db", db_type: "memory" });
        expect(db_add).toHaveBeenCalledOnce();
    });

    it("does nothing if not logged in", async () => {
        username.value = "";
        const { addDatabase } = useDbStore();
        await addDatabase({ name: "test_db", db_type: "memory" });
        expect(db_add).not.toHaveBeenCalled();
    });
});
