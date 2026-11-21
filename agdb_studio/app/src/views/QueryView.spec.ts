import { describe, it, expect, vi } from "vitest";
import QueryView from "./QueryView.vue";
import { mount } from "@vue/test-utils";
import { nextTick } from "vue";

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

vi.mock(
  "@agdb-studio/dashboard/src/components/PinToDashboardButton.vue",
  () => ({
    default: {
      name: "PinToDashboardButton",
      props: ["owner", "db", "queryId"],
      template: '<button class="pin">{{ queryId }}</button>',
    },
  }),
);

const { builderQueries } = vi.hoisted(() => ({
  builderQueries: { value: "" },
}));

vi.mock("@agdb-studio/query/src/components/view/QueryView.vue", () => ({
  default: {
    name: "QueryView",
    props: ["owner", "db"],
    setup(_: unknown, { expose }: { expose: (e: object) => void }) {
      expose({ queryId: builderQueries.value });
    },
    template:
      '<div class="query-view">Database {{ owner }}/{{ db }} query</div>',
  },
}));

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

  it("passes the builder's query id to the pin button on the user query route", async () => {
    builderQueries.value = "query-1";
    useRoute.mockReturnValue({
      name: "query",
      params: { owner: "o", db: "d" },
    });
    const wrapper = mount(QueryView, {});
    await nextTick();
    expect(wrapper.find(".pin").text()).toBe("query-1");
    useRoute.mockReturnValue({
      name: "admin-query",
      params: { owner: "o", db: "d" },
    });
    expect(mount(QueryView, {}).find(".pin").exists()).toBe(false);
  });
});
