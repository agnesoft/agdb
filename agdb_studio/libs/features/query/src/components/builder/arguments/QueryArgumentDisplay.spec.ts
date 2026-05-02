import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import QueryArgumentDisplay from "./QueryArgumentDisplay.vue";
import type { QueryArguments } from "../../../mock/queryApiMock";
import type { QueryStep } from "../../../composables/types";

const VALUE_ARGS: QueryArguments = {
  fields: [
    { options: ["string", "unsigned", "signed"] },
    { options: ["string", "unsigned", "signed"] },
  ],
  repeatable: true,
};

const NO_VALUE_ARGS: QueryArguments = {
  fields: [{ options: ["asc", "desc"] }],
  repeatable: false,
};

const makeStep = (overrides: Partial<QueryStep> = {}): QueryStep => ({
  id: "step-1",
  type: "values",
  ...overrides,
});

describe("QueryArgumentDisplay", () => {
  it("shows placeholder when step has no args", () => {
    const wrapper = mount(QueryArgumentDisplay, {
      props: { arguments: VALUE_ARGS, step: makeStep() },
    });
    expect(wrapper.find(".arg-display").text()).toBe("(…)");
  });

  it("adds placeholder class when no args", () => {
    const wrapper = mount(QueryArgumentDisplay, {
      props: { arguments: VALUE_ARGS, step: makeStep() },
    });
    expect(wrapper.find(".arg-display").classes()).toContain("placeholder");
  });

  it("does not have placeholder class when args exist", () => {
    const step = makeStep({
      args: [
        [
          { selectedOption: "string", value: "hello" },
          { selectedOption: "unsigned", value: "42" },
        ],
      ],
    });
    const wrapper = mount(QueryArgumentDisplay, {
      props: { arguments: VALUE_ARGS, step },
    });
    expect(wrapper.find(".arg-display").classes()).not.toContain("placeholder");
  });

  it("displays values in parentheses for a single entry", () => {
    const step = makeStep({
      args: [
        [
          { selectedOption: "string", value: "hello" },
          { selectedOption: "unsigned", value: "42" },
        ],
      ],
    });
    const wrapper = mount(QueryArgumentDisplay, {
      props: { arguments: VALUE_ARGS, step },
    });
    expect(wrapper.find(".arg-display").text()).toBe("(hello, 42)");
  });

  it("shows multiple entries comma-separated", () => {
    const step = makeStep({
      args: [
        [
          { selectedOption: "string", value: "a" },
          { selectedOption: "unsigned", value: "1" },
        ],
        [
          { selectedOption: "signed", value: "b" },
          { selectedOption: "string", value: "2" },
        ],
      ],
    });
    const wrapper = mount(QueryArgumentDisplay, {
      props: { arguments: VALUE_ARGS, step },
    });
    expect(wrapper.find(".arg-display").text()).toBe("(a, 1), (b, 2)");
  });

  it("omits fields with no value entered", () => {
    const step = makeStep({
      args: [
        [
          { selectedOption: "string", value: "hello" },
          { selectedOption: "unsigned", value: undefined },
        ],
      ],
    });
    const wrapper = mount(QueryArgumentDisplay, {
      props: { arguments: VALUE_ARGS, step },
    });
    expect(wrapper.find(".arg-display").text()).toBe("(hello)");
  });

  it("omits options that take no value entirely", () => {
    const step = makeStep({
      type: "orderBy",
      args: [[{ selectedOption: "asc", value: undefined }]],
    });
    const wrapper = mount(QueryArgumentDisplay, {
      props: { arguments: NO_VALUE_ARGS, step },
    });
    // asc takes no value so nothing to display — shows placeholder
    expect(wrapper.find(".arg-display").text()).toBe("()");
  });

  it("shows empty parentheses when all fields have no value type", () => {
    const step = makeStep({
      type: "orderBy",
      args: [[{ selectedOption: "desc", value: undefined }]],
    });
    const wrapper = mount(QueryArgumentDisplay, {
      props: { arguments: NO_VALUE_ARGS, step },
    });
    expect(wrapper.find(".arg-display").text()).toBe("()");
  });
});
