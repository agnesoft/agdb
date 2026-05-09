import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import QueryStep from "./QueryStep.vue";
import type { QueryStep as QueryStepType } from "../../composables/types";
import { nextTick, ref } from "vue";

const { deleteQueryStep, updateQueryStep } = vi.hoisted(() => ({
  deleteQueryStep: vi.fn(),
  updateQueryStep: vi.fn(),
}));

vi.mock("../../composables/queryStore", () => ({
  useQueryStore: () => ({
    deleteQueryStep,
    updateQueryStep,
  }),
}));

const globalProvide = {
  provide: {
    queryId: ref("test-query-1"),
    activeTab: ref("exec"),
  },
};

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
      global: globalProvide,
    });
    expect(wrapper.find(".query-step").exists()).toBe(true);
    expect(wrapper.find(".label").text()).toContain("select");
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
      expect(wrapper.find(".label").text()).toContain(type);
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
      global: globalProvide,
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

  it("renders QueryArgumentDisplay for a step type with arguments", () => {
    const step: QueryStepType = {
      id: "step-1",
      type: "from",
      args: [[{ selectedOption: "signed", value: "1" }]],
    };
    const wrapper = mount(QueryStep, {
      props: { step },
      global: globalProvide,
    });
    expect(wrapper.find(".arg-display").exists()).toBe(true);
  });

  it("does not render QueryArgumentDisplay for a step type without arguments", () => {
    const step: QueryStepType = { id: "step-1", type: "select" };
    const wrapper = mount(QueryStep, {
      props: { step },
      global: globalProvide,
    });
    expect(wrapper.find(".arg-display").exists()).toBe(false);
  });

  it("opens the arg editor popup when display is clicked", async () => {
    const step: QueryStepType = {
      id: "step-1",
      type: "from",
      args: [[{ selectedOption: "signed", value: "1" }]],
    };
    const wrapper = mount(QueryStep, {
      props: { step },
      global: globalProvide,
    });
    expect(wrapper.find(".arg-editor-popup").exists()).toBe(false);
    await wrapper.find(".arg-display-trigger").trigger("click");
    expect(wrapper.find(".arg-editor-popup").exists()).toBe(true);
  });

  it("starts in edit mode automatically for a new step with no args", () => {
    const step: QueryStepType = { id: "step-1", type: "from" };
    const wrapper = mount(QueryStep, {
      props: { step },
      global: globalProvide,
    });
    expect(wrapper.find(".arg-editor-popup").exists()).toBe(true);
  });

  it("focuses first argument dropdown for a new step with arguments", async () => {
    const step: QueryStepType = { id: "step-1", type: "from" };
    const wrapper = mount(QueryStep, {
      props: { step },
      global: globalProvide,
      attachTo: document.body,
    });

    await nextTick();
    await nextTick();

    const firstTrigger = wrapper.find(".arg-select-trigger");
    expect(firstTrigger.exists()).toBe(true);
    expect(document.activeElement).toBe(firstTrigger.element);

    wrapper.unmount();
  });
});
