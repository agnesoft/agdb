import { shallowMount } from "@vue/test-utils";
import AgdbTableRow from "./AgdbTableRow.vue";
import { columnsMap } from "@/tests/tableMocks";

describe("TableRow", () => {
    it("should render", () => {
        const wrapper = shallowMount(AgdbTableRow, {
            props: {
                columns: columnsMap,
                row: {
                    role: "admin",
                    name: "admin/app3",
                    db_type: "file",
                    size: 50,
                    backup: 0,
                },
            },
        });
        expect(wrapper.text()).toContain("admin");
    });
});
