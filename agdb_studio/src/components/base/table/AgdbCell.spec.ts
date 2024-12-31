import { describe, beforeEach, vi, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import AgdbCell from "./AgdbCell.vue";
import { dbColumns } from "@/composables/db/dbConfig";

describe("AgdbCell", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("should render the cell", () => {
        const wrapper = mount(AgdbCell, {
            props: {
                cellKey: "test",
            },
            provide: {
                columns: { value: dbColumns },
                row: { value: { test: "test" } },
            },
        });

        expect(wrapper.find(".agdb-cell").exists()).toBe(true);
        expect(wrapper.find(".agdb-cell").text()).toBe("test");
    });
});
