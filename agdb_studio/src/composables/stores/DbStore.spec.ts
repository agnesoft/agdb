import { useDbList } from "./DbStore";
import { db_list, db_add } from "@/tests/apiMock";

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

const { username } = vi.hoisted(() => {
    return {
        username: { value: "test_user" },
    };
});
vi.mock("../user/account", () => {
    return {
        useAccount: () => {
            return {
                username,
            };
        },
    };
});

describe("DbStore", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("fetches databases when called", async () => {
        const { databases, fetchDatabases } = useDbList();
        await fetchDatabases();
        expect(databases.value).toHaveLength(2);
    });

    it("adds a database when called", async () => {
        expect(db_add).not.toHaveBeenCalled();
        const { addDatabase } = useDbList();
        await addDatabase({ name: "test_db", db_type: "memory" });
        expect(db_add).toHaveBeenCalledOnce();
    });

    it("does nothing if not logged in", async () => {
        username.value = "";
        const { addDatabase } = useDbList();
        await addDatabase({ name: "test_db", db_type: "memory" });
        expect(db_add).not.toHaveBeenCalled();
    });
});
