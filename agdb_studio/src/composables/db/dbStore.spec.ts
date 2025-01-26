import { useDbStore } from "./dbStore";
import { describe, beforeEach, vi, it, expect } from "vitest";

const { username, admin, dbAdd, dbList } = vi.hoisted(() => {
    return {
        username: { value: "test_user" },
        admin: { value: false },
        dbAdd: vi.fn().mockResolvedValue({
            data: {
                name: "test_db",
                db_type: "memory",
                role: "admin",
                size: 2656,
                backup: 0,
            },
        }),
        dbList: vi.fn().mockResolvedValue({
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
        }),
    };
});
vi.mock("@/composables/profile/account", () => {
    return {
        useAccount: () => {
            return {
                username,
                admin,
            };
        },
    };
});

vi.mock("./dbActions", () => {
    return {
        dbAdd,
        dbList,
    };
});

describe("DbStore", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("fetches databases when called", async () => {
        const { databases, fetchDatabases } = useDbStore();
        await fetchDatabases();
        expect(dbList).toHaveBeenCalledOnce();
        expect(databases.value).toHaveLength(2);
    });

    it("adds a database when called", async () => {
        expect(dbAdd).not.toHaveBeenCalled();
        const { addDatabase } = useDbStore();
        await addDatabase({ name: "test_db", db_type: "memory" });
        expect(dbAdd).toHaveBeenCalledOnce();
    });

    it("does nothing if not logged in", async () => {
        username.value = "";
        const { addDatabase } = useDbStore();
        await addDatabase({ name: "test_db", db_type: "memory" });
        expect(dbList).not.toHaveBeenCalled();
    });
});
