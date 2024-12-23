import { shallowMount } from "@vue/test-utils";
import TableHeader from "./TableHeader.vue";

describe("TableHeader", () => {
    it("should render", () => {
        const wrapper = shallowMount(TableHeader, {
            props: {
                tableKey: "table",
            },
        });
        expect(wrapper.exists()).toBe(true);
    });
});
