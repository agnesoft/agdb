import { mount } from "@vue/test-utils";
import { describe, beforeEach, vi, it, expect } from "vitest";
import DbDetails from "./DbDetails.vue";
import { db_user_list, db_user_add, db_user_remove } from "@/tests/apiMock";

describe("DbDetails", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("should render users", async () => {
        db_user_list.mockResolvedValue({
            data: [
                {
                    username: "test",
                    role: "read",
                },
            ],
        });
        const wrapper = mount(DbDetails, {
            props: {
                db: {
                    owner: "test",
                    db: "test",
                },
            },
        });
        await wrapper.vm.$nextTick();
        expect(wrapper.text()).toContain("test");
        expect(wrapper.text()).toContain("(R)");
    });
});
