import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import QueryStep from "./QueryStep.vue";
import type { QueryStep as QueryStepType } from "../../composables/types";
import { ref } from "vue";

const { deleteQueryStep } = vi.hoisted(() => ({ deleteQueryStep: vi.fn() }));

vi.mock("../../composables/queryStore", () => ({
  useQueryStore: () => ({
    deleteQueryStep: deleteQueryStep,
  }),
}));

describe("QueryStep", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });
  it("renders the step type", () => {
    const step: QueryStepType = {
      id: "step-1",
      type: "select",
    };
    const wrapper = mount(QueryStep, {
      props: { step },
      global: {
        provide: {
          queryId: ref("test-query-1"),
          activeTab: ref("exec"),
        },
      },
    });
    expect(wrapper.find(".query-step").exists()).toBe(true);
    expect(wrapper.find(".label").text()).toBe("select");
  });

  it("renders different step types correctly", () => {
    const stepTypes: QueryStepType["type"][] = ["select", "insert", "search"];

    stepTypes.forEach((type) => {
      const step: QueryStepType = {
        id: `step-${type}`,
        type: type,
      };
      const wrapper = mount(QueryStep, {
        props: { step },
      });
      expect(wrapper.find(".label").text()).toBe(type);
    });
  });

  it("applies correct styling classes", () => {
    const step: QueryStepType = {
      id: "step-1",
      type: "select",
    };
    const wrapper = mount(QueryStep, {
      props: { step },
    });
    expect(wrapper.find(".query-step").exists()).toBe(true);
    expect(wrapper.find(".label").exists()).toBe(true);
  });

  it("deletes step on delete button click", async () => {
    const step: QueryStepType = {
      id: "step-1",
      type: "select",
    };
    const wrapper = mount(QueryStep, {
      props: { step },
      global: {
        provide: {
          queryId: ref("test-query-1"),
          activeTab: ref("exec"),
        },
      },
    });
    const deleteButton = wrapper.find(".remove-step-button");
    await deleteButton.trigger("click");
    expect(deleteQueryStep).toHaveBeenCalledWith(
      "test-query-1",
      "exec",
      "step-1",
    );
  });

  it("handles missing queryId gracefully on delete", async () => {
    const step: QueryStepType = {
      id: "step-1",
      type: "select",
    };
    const wrapper = mount(QueryStep, {
      props: { step },
      global: {
        provide: {
          queryId: ref(undefined),
          activeTab: ref("exec"),
        },
      },
    });
    const deleteButton = wrapper.find(".remove-step-button");
    await deleteButton.trigger("click");
    expect(deleteQueryStep).not.toHaveBeenCalled();
  });
});
