import { shallowMount } from "@vue/test-utils";
import TableRow from "./TableRow.vue";

describe("TableRow", () => {
    it("should render", () => {
        const wrapper = shallowMount(TableRow, {
            props: {
                columns: new Map([
                    ["name", { key: "name", title: "Name" }],
                    ["age", { key: "age", title: "Age" }],
                    ["job", { key: "job", title: "Job" }],
                ]),
                row: {
                    name: "John Doe",
                    age: 30,
                    job: "Developer",
                },
            },
        });
        expect(wrapper.exists()).toBe(true);
    });
});
