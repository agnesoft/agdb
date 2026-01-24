import { describe, it, expect, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import QueryBuilder from "./QueryBuilder.vue";
import { useQueryStore } from "../../composables/queryStore";
import { ref } from "vue";

describe("QueryBuilder", () => {
  let queryStore: ReturnType<typeof useQueryStore>;

  beforeEach(() => {
    queryStore = useQueryStore();
    queryStore.clearQueries();
  });

  it("renders the query builder", () => {
    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref("test-query-1"),
        },
      },
    });
    expect(wrapper.find(".query-builder").exists()).toBe(true);
    expect(wrapper.find(".query-input").exists()).toBe(true);
    expect(wrapper.find("button.button-primary").exists()).toBe(true);
  });

  it("renders with exec tab styling", () => {
    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref("test-query-1"),
          activeTab: ref("exec"),
        },
      },
    });
    expect(wrapper.find(".run-query-button").classes()).toContain(
      "button-primary",
    );
  });

  it("renders with exec_mut tab styling", () => {
    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref("test-query-1"),
          activeTab: ref("exec_mut"),
        },
      },
    });
    expect(wrapper.find(".run-query-button").classes()).toContain(
      "button-warning",
    );
  });

  it("renders with context tab styling", () => {
    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref("test-query-1"),
          activeTab: ref("context"),
        },
      },
    });
    expect(wrapper.find(".run-query-button").classes()).toContain(
      "button-primary",
    );
  });

  it("adds a new step when confirm-step is emitted", async () => {
    const queryId = "test-query-3";
    queryStore.addQuery({ id: queryId });

    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref(queryId),
        },
      },
    });

    const stepInput = wrapper.findComponent({ name: "QueryStepInput" });
    await stepInput.vm.$emit("confirm-step", "select");

    const query = queryStore.getQuery(queryId);
    expect(query?.value.steps.exec.length).toBe(1);
    expect(query?.value.steps.exec[0]?.type).toBe("select");
  });

  it("handles missing queryId gracefully", () => {
    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref(undefined),
        },
      },
    });
    expect(wrapper.find(".query-builder").exists()).toBe(true);
  });

  it("shows QueryStepInput component", () => {
    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref("test-query-4"),
        },
      },
    });
    const stepInput = wrapper.findComponent({ name: "QueryStepInput" });
    expect(stepInput.exists()).toBe(true);
  });

  it("can remove all steps", async () => {
    const queryId = "test-query-5";
    queryStore.addQuery({ id: queryId });

    const wrapper = mount(QueryBuilder, {
      global: {
        provide: {
          queryId: ref(queryId),
        },
      },
    });

    const stepInput = wrapper.findComponent({ name: "QueryStepInput" });
    await stepInput.vm.$emit("confirm-step", "select");
    await stepInput.vm.$emit("confirm-step", "search");

    let query = queryStore.getQuery(queryId);
    expect(query?.value.steps.exec.length).toBe(2);

    const removeButton = wrapper.find(".remove-button");
    await removeButton.trigger("click");

    query = queryStore.getQuery(queryId);
    expect(query?.value.steps.exec.length).toBe(0);
  });
});
