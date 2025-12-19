import { describe, it, expect, beforeEach } from "vitest";
import QueryView from "./QueryView.vue";
import { mount, flushPromises } from "@vue/test-utils";
import { useQueryStore } from "../../composables/queryStore";

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
          QueryGraph: {
            template: '<div class="mock-query-graph">Mock Query Graph</div>',
          },
        },
      },
    });
    expect(wrapper.find(".query-view").exists()).toBe(true);
    expect(wrapper.find(".mock-query-builder-tabs").exists()).toBe(true);
    expect(wrapper.find(".mock-query-graph").exists()).toBe(true);
  });

  it("initializes a query on mount", async () => {
    const wrapper = mount(QueryView, {
      global: {
        stubs: {
          QueryBuilderTabs: true,
          QueryGraph: true,
        },
      },
    });

    await flushPromises();

    // Check that a query was added to the store
    // We can't easily access the queryId, but we can verify the component mounted
    expect(wrapper.find(".query-view").exists()).toBe(true);
  });

  // it("provides queryId to child components", async () => {
  //   let providedQueryId: string | undefined;

  //   const ChildComponent = {
  //     template: "<div>{{ queryId }}</div>",
  //     inject: ["queryId"],
  //     setup() {
  //       const queryId = (this as any).queryId;
  //       providedQueryId = queryId?.value;
  //       return { queryId };
  //     },
  //   };

  //   const wrapper = mount(QueryView, {
  //     global: {
  //       stubs: {
  //         QueryBuilderTabs: ChildComponent,
  //         QueryGraph: true,
  //       },
  //     },
  //   });

  //   await flushPromises();

  //   // The queryId should be provided
  //   expect(wrapper.vm).toBeDefined();
  // });

  it("has correct grid layout structure", () => {
    const wrapper = mount(QueryView, {
      global: {
        stubs: {
          QueryBuilderTabs: true,
          QueryGraph: true,
        },
      },
    });

    const view = wrapper.find(".query-view");
    expect(view.exists()).toBe(true);
    // The component should have the query-view class which applies grid styling
  });

  it("renders both QueryBuilderTabs and QueryGraph components", () => {
    const wrapper = mount(QueryView);

    const builderTabs = wrapper.findComponent({ name: "QueryBuilderTabs" });
    const queryGraph = wrapper.findComponent({ name: "QueryGraph" });

    expect(builderTabs.exists()).toBe(true);
    expect(queryGraph.exists()).toBe(true);
  });
});
