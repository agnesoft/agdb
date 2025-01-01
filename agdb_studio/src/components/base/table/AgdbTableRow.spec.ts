import { mount } from "@vue/test-utils";
import AgdbTableRow from "./AgdbTableRow.vue";
import { columnsMap } from "@/tests/tableMocks";
import { describe, it, expect } from "vitest";
import { INJECT_KEY_COLUMNS } from "@/composables/table/constants";

describe("TableRow", () => {
    it("should render", () => {
        const wrapper = mount(AgdbTableRow, {
            props: {
                columns: columnsMap,
                row: {
                    role: "admin",
                    owner: "admin",
                    db: "app3",
                    db_type: "file",
                    size: 50,
                    backup: 0,
                },
            },

            global: {
                provide: {
                    [INJECT_KEY_COLUMNS]: { value: columnsMap },
                },
            },
        });
        expect(wrapper.text()).toContain("admin");
        expect(wrapper.text()).toContain("app3");
    });
});
