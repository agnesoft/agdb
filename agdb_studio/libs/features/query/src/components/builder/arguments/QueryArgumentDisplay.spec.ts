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

  it("displays option shortcuts with values for a single entry", () => {
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
    expect(wrapper.find(".arg-display").text()).toBe("(s, hello, u, 42)");
  });

  it("shows multiple entries with shortcuts comma-separated", () => {
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
    expect(wrapper.find(".arg-display").text()).toBe(
      "(s, a, u, 1), (i, b, s, 2)",
    );
  });

  it("keeps option shortcut even when value is missing", () => {
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
    expect(wrapper.find(".arg-display").text()).toBe("(s, hello, u)");
  });

  it("shows shortcut for options that take no value", () => {
    const step = makeStep({
      type: "orderBy",
      args: [[{ selectedOption: "asc", value: undefined }]],
    });
    const wrapper = mount(QueryArgumentDisplay, {
      props: { arguments: NO_VALUE_ARGS, step },
    });
    expect(wrapper.find(".arg-display").text()).toBe("(↑)");
  });

  it("shows symbolic shortcut for no-value direction options", () => {
    const step = makeStep({
      type: "orderBy",
      args: [[{ selectedOption: "desc", value: undefined }]],
    });
    const wrapper = mount(QueryArgumentDisplay, {
      props: { arguments: NO_VALUE_ARGS, step },
    });
    expect(wrapper.find(".arg-display").text()).toBe("(↓)");
  });
});
