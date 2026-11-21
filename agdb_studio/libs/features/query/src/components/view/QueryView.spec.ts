import { describe, it, expect, beforeEach, vi } from "vitest";
import QueryView from "./QueryView.vue";
import { mount, flushPromises } from "@vue/test-utils";
import { useQueryStore } from "../../composables/queryStore";

// The results (graph, table, actions) are tested on their own.
vi.mock("../results/QueryResults.vue", () => ({
  default: {
    name: "QueryResults",
    template: "<div class='mock-query-results'>Mock Query Results</div>",
  },
}));

vi.mock("../builder/QueryBuilderTabs.vue", () => ({
  default: {
    name: "QueryBuilderTabs",
    props: ["tabs"],
    template:
      "<div class='mock-query-builder-tabs'>Mock Query Builder Tabs</div>",
  },
}));
describe("QueryView", () => {
  let queryStore: ReturnType<typeof useQueryStore>;

  beforeEach(() => {
    queryStore = useQueryStore();
    queryStore.clearQueries();
  });

  it("renders the query view", () => {
    const wrapper = mount(QueryView, {
      global: {
        stubs: {
          QueryBuilderTabs: {
            template:
              '<div class="mock-query-builder-tabs">Mock Query Builder Tabs</div>',
          },
          QueryResults: {
            template:
              '<div class="mock-query-results">Mock Query Results</div>',
          },
        },
      },
    });
    expect(wrapper.find(".query-workspace").exists()).toBe(true);
    expect(wrapper.find(".mock-query-builder-tabs").exists()).toBe(true);
    expect(wrapper.find(".mock-query-results").exists()).toBe(true);
  });

  it("initializes a query on mount", async () => {
    const wrapper = mount(QueryView, {
      global: {
        stubs: {
          QueryBuilderTabs: true,
          QueryResults: true,
        },
      },
    });

    await flushPromises();

    // Check that a query was added to the store
    // We can't easily access the queryId, but we can verify the component mounted
    expect(wrapper.find(".query-workspace").exists()).toBe(true);
  });

  it("has correct grid layout structure", () => {
    const wrapper = mount(QueryView, {
      global: {
        stubs: {
          QueryBuilderTabs: true,
          QueryResults: true,
        },
      },
    });

    const view = wrapper.find(".query-workspace");
    expect(view.exists()).toBe(true);
    // The component should have the query-workspace class which applies grid styling
  });

  it("renders both QueryBuilderTabs and QueryResults components", () => {
    const wrapper = mount(QueryView);

    const builderTabs = wrapper.findComponent({ name: "QueryBuilderTabs" });
    const queryResults = wrapper.findComponent({ name: "QueryResults" });

    expect(builderTabs.exists()).toBe(true);
    expect(queryResults.exists()).toBe(true);
  });

  it("creates its own query entry and exposes its id", async () => {
    vi.spyOn(crypto, "randomUUID").mockReturnValueOnce("a-b-c-d-e");
    const wrapper = mount(QueryView, {
      global: { stubs: { QueryBuilderTabs: true, QueryResults: true } },
    });
    await flushPromises();
    const vm = wrapper.vm as unknown as { queryId: string };
    expect(vm.queryId).toBe("query-a-b-c-d-e");
    expect(queryStore.getQuery("query-a-b-c-d-e")).toBeDefined();
  });

  it("works on a given query entry and passes the tabs to the builder", async () => {
    queryStore.addQuery({ id: "given" });
    const wrapper = mount(QueryView, {
      props: { queryId: "given", tabs: ["exec", "context"] },
      global: { stubs: { QueryResults: true } },
    });
    await flushPromises();
    const vm = wrapper.vm as unknown as { queryId: string };
    expect(vm.queryId).toBe("given");
    expect(
      wrapper.findComponent({ name: "QueryBuilderTabs" }).props("tabs"),
    ).toEqual(["exec", "context"]);
  });
});
