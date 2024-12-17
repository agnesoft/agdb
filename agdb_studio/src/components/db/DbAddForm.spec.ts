import DbAddForm from "./DbAddForm.vue";
import { mount } from "@vue/test-utils";

const { addDatabase } = vi.hoisted(() => {
    return {
        addDatabase: vi.fn(),
    };
});

vi.mock("@/composables/stores/DbStore", () => {
    return {
        useDbList: () => {
            return {
                addDatabase,
            };
        },
    };
});

describe("DbAddForm", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("should add a database when user submits", async () => {
        expect(addDatabase).not.toHaveBeenCalled();
        const wrapper = mount(DbAddForm);
        await wrapper.find("input").setValue("test_db");
        await wrapper.find("select").setValue("memory");
        await wrapper.find("form").trigger("submit");
        await wrapper.vm.$nextTick();
        expect(addDatabase).toHaveBeenCalledOnce();
    });
    it("should add a database when user clicks submit button", async () => {
        expect(addDatabase).not.toHaveBeenCalled();
        const wrapper = mount(DbAddForm);
        await wrapper.find("input").setValue("test_db");
        await wrapper.find("select").setValue("memory");
        await wrapper.find("button[type=submit]").trigger("click");
        await wrapper.vm.$nextTick();
        expect(addDatabase).toHaveBeenCalledOnce();
    });
});
