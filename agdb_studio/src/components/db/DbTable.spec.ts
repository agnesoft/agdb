import { shallowMount } from "@vue/test-utils";
import DbTable from "./DbTable.vue";
import { describe, it, expect } from "vitest";

describe("DbTable", () => {
    it("should render", () => {
        const wrapper = shallowMount(DbTable, {
            props: {
                tableKey: "table",
            },
        });
        expect(wrapper.exists()).toBe(true);
    });
});
