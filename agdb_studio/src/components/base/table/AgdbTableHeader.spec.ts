import { shallowMount } from "@vue/test-utils";
import AgdbTableHeader from "./AgdbTableHeader.vue";

describe("TableHeader", () => {
    it("should render", () => {
        const wrapper = shallowMount(AgdbTableHeader, {
            props: {
                tableKey: "table",
            },
        });
        expect(wrapper.exists()).toBe(true);
    });
});
