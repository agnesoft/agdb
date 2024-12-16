import DbView from "./DbView.vue";
import { mount } from "@vue/test-utils";

const { databases, fetchDatabases } = vi.hoisted(() => {
    return {
        databases: [
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

        fetchDatabases: vi.fn(),
    };
});

vi.mock("@/composables/stores/DbStore", () => {
    return {
        useDbList: () => {
            return {
                databases,
                fetchDatabases,
            };
        },
    };
});

describe("DbView", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("should render a list of databases", () => {
        const wrapper = mount(DbView);
        expect(wrapper.text()).toContain("test_db");
        expect(wrapper.text()).toContain("test_db2");
    });
});
