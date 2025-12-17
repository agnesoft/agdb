import { describe, it, expect, vi } from "vitest";
import QueryView from "./QueryView.vue";
import { mount } from "@vue/test-utils";

const { useRoute } = vi.hoisted(() => ({ useRoute: vi.fn() }));

vi.mock("vue-router", () => ({
  useRoute,
}));

vi.mock("@agdb-studio/db/src/composables/dbStore", () => {
  return {
    useDbStore: () => ({
      getDbName: vi
        .fn()
        .mockImplementation(
          (db: { owner: string; db: string }) => `${db.owner}/${db.db}`,
        ),
    }),
  };
});

describe("QueryView", () => {
  it("renders the query view", () => {
    useRoute.mockReturnValueOnce({
      params: {
        owner: "test_owner",
        db: "test_db",
      },
    });
    const wrapper = mount(QueryView, {});
    expect(wrapper.text()).toContain("Database test_owner/test_db query");
  });
  it("should handle when route params are not strings", () => {
    useRoute.mockReturnValueOnce({
      params: {
        owner: undefined,
        db: [],
      },
    });
    const wrapper = mount(QueryView, {});
    expect(wrapper.text()).toContain("Database / query");
  });
});
