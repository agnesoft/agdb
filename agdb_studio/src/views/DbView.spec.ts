import DbView from "./DbView.vue";
import { mount, shallowMount } from "@vue/test-utils";

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

vi.mock("@/composables/db/DbStore", () => {
    return {
        useDbStore: () => {
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
    it("should render databases when the page view loads", () => {
        const wrapper = shallowMount(DbView);
        expect(wrapper.html()).toContain("db-add-form-stub");
        expect(wrapper.html()).toContain("db-table-stub");
    });
    it("should fetch databases when the page view loads", () => {
        expect(fetchDatabases).not.toHaveBeenCalled();
        mount(DbView);
        expect(fetchDatabases).toHaveBeenCalledOnce();
    });
    it("should render a message when there are no databases", () => {
        databases.length = 0;
        const wrapper = mount(DbView);
        expect(wrapper.text()).toContain("No databases found");
    });
    it("should refresh databases when user clicks refresh button", async () => {
        expect(fetchDatabases).not.toHaveBeenCalled();
        const wrapper = mount(DbView);
        await wrapper.find("button").trigger("click");
        await wrapper.vm.$nextTick();
        expect(fetchDatabases).toHaveBeenCalledTimes(2);
    });
});
