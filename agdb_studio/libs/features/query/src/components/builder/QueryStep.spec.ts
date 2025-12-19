import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import QueryStep from "./QueryStep.vue";
import type { QueryStep as QueryStepType } from "../../composables/types";

describe("QueryStep", () => {
  it("renders the step type", () => {
    const step: QueryStepType = {
      id: "step-1",
      type: "select",
    };
    const wrapper = mount(QueryStep, {
      props: { step },
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

  // it("renders with values when provided", () => {
  //   const step: QueryStepType = {
  //     id: "step-1",
  //     type: "select",
  //   };
  //   const wrapper = mount(QueryStep, {
  //     props: { step },
  //   });
  //   expect(wrapper.find(".label").text()).toBe("select");
  // });
});
