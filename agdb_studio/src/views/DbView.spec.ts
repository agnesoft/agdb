import DbView from "./DbView.vue";
import { shallowMount } from "@vue/test-utils";

describe("DbView", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("should render a list of databases", () => {
        const wrapper = shallowMount(DbView);
        expect(wrapper.html()).toContain("db-list-stub");
    });
});
