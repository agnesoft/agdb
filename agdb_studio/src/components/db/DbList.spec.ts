import DbList from "./DbList.vue";
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

vi.mock("@/composables/DbStore", () => {
    return {
        useDbList: () => {
            return {
                databases,
                fetchDatabases,
            };
        },
    };
});

describe("DbList", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("should render databases when the page view loads", () => {
        const wrapper = mount(DbList);
        const rows = wrapper.findAll("li");
        expect(rows).toHaveLength(2);
        expect(rows[0].text()).toContain("test_db");
        expect(rows[1].text()).toContain("test_db2");
    });
    it("should fetch databases when the page view loads", () => {
        expect(fetchDatabases).not.toHaveBeenCalled();
        mount(DbList);
        expect(fetchDatabases).toHaveBeenCalledOnce();
    });
    it("should render a message when there are no databases", () => {
        databases.length = 0;
        const wrapper = mount(DbList);
        expect(wrapper.text()).toContain("No databases found");
    });
    it("should refresh databases when user clicks refresh button", async () => {
        expect(fetchDatabases).not.toHaveBeenCalled();
        const wrapper = mount(DbList);
        await wrapper.find("button").trigger("click");
        await wrapper.vm.$nextTick();
        expect(fetchDatabases).toHaveBeenCalledTimes(2);
    });
});
