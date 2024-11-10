import DbListRow from "./DbListRow.vue";
import { mount } from "@vue/test-utils";

describe("DbListRow", () => {
    it("should render row with correct data when loaded", () => {
        const wrapper = mount(DbListRow, {
            propsData: {
                db: {
                    name: "test_db",
                    db_type: "memory",
                    role: "admin",
                    size: 2656,
                    backup: 0,
                },
            },
        });
        expect(wrapper.text()).toContain("test_db");
        expect(wrapper.text()).toContain("memory");
        expect(wrapper.text()).toContain("admin");
        expect(wrapper.text()).toContain("2656");
    });
});
