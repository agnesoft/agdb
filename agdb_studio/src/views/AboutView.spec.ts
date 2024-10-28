import { mount } from "@vue/test-utils";
import AboutView from "./AboutView.vue";

describe("AboutView", () => {
    it("renders properly", () => {
        const wrapper = mount(AboutView);
        expect(wrapper.text()).toContain("about");
    });
});
