import { shallowMount } from "@vue/test-utils";
import AgdbTable from "./AgdbTable.vue";

describe("AgdbTable", () => {
    it("should render", () => {
        const wrapper = shallowMount(AgdbTable, {
            props: {
                name: "table",
            },
        });
        expect(wrapper.exists()).toBe(true);
    });
});
