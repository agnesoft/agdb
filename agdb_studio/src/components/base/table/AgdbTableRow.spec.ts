import { shallowMount } from "@vue/test-utils";
import AgdbTableRow from "./AgdbTableRow.vue";
import { columnsMap } from "@/tests/tableMocks";
import { describe, beforeEach, vi, it, expect } from "vitest";

describe("TableRow", () => {
    it("should render", () => {
        const wrapper = shallowMount(AgdbTableRow, {
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
        });
        expect(wrapper.text()).toContain("admin");
        expect(wrapper.text()).toContain("app3");
    });
});
