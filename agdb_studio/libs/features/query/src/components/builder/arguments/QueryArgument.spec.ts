import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { ref } from "vue";
import QueryArgument from "./QueryArgument.vue";
import type { QueryArguments } from "../../../mock/queryApiMock";
import type { QueryStep } from "../../../composables/types";

const { updateQueryStep } = vi.hoisted(() => ({ updateQueryStep: vi.fn() }));

vi.mock("../../../composables/queryStore", () => ({
  useQueryStore: () => ({ updateQueryStep }),
}));

const REPEATABLE_ARGS: QueryArguments = {
  fields: [
    {
      options: [
        "string",
        "unsigned",
        "signed",
        "boolean",
        "float",
        "string[]",
        "unsigned[]",
        "signed[]",
        "boolean[]",
        "float[]",
      ],
    },
    {
      options: [
        "string",
        "unsigned",
        "signed",
        "boolean",
        "float",
        "string[]",
        "unsigned[]",
        "signed[]",
        "boolean[]",
        "float[]",
      ],
    },
  ],
  repeatable: true,
};

const SINGLE_ARGS: QueryArguments = {
  fields: [{ options: ["signed", "string"] }],
  repeatable: false,
};

const NO_VALUE_ARGS: QueryArguments = {
  fields: [{ options: ["asc", "desc"] }],
  repeatable: false,
};

const makeStep = (overrides: Partial<QueryStep> = {}): QueryStep => ({
  id: "step-1",
  type: "from",
  ...overrides,
});

const globalProvide = {
  provide: {
    queryId: ref("q1"),
    activeTab: ref("exec"),
  },
};

describe("QueryArgument", () => {
  beforeEach(() => vi.clearAllMocks());

  it("commits an initial empty entry on mount when no args", () => {
    mount(QueryArgument, {
      props: { arguments: SINGLE_ARGS, step: makeStep() },
      global: globalProvide,
    });
    expect(updateQueryStep).toHaveBeenCalledOnce();
    const committed = updateQueryStep.mock.calls[0]?.[2];
    expect(committed.args).toHaveLength(1);
    expect(committed.args[0]).toHaveLength(1);
    expect(committed.args[0][0].selectedOption).toBe("signed");
  });

  it("renders one dropdown per field", () => {
    const step = makeStep({
      args: [[{ selectedOption: "signed", value: undefined }]],
    });
    const wrapper = mount(QueryArgument, {
      props: { arguments: SINGLE_ARGS, step },
      global: globalProvide,
    });
    expect(wrapper.findAll(".arg-field")).toHaveLength(1);
    expect(
      wrapper.findAllComponents({ name: "QueryArgumentDropdown" }),
    ).toHaveLength(1);
  });

  it("renders an input when selected option takes a value", () => {
    const step = makeStep({
      type: "from",
      args: [[{ selectedOption: "signed", value: "1" }]],
    });
    const wrapper = mount(QueryArgument, {
      props: { arguments: SINGLE_ARGS, step },
      global: globalProvide,
    });
    expect(wrapper.find(".arg-input").exists()).toBe(true);
  });

  it("does not render an input when selected option takes no value", () => {
    const step = makeStep({
      type: "orderBy",
      args: [[{ selectedOption: "asc", value: undefined }]],
    });
    const wrapper = mount(QueryArgument, {
      props: { arguments: NO_VALUE_ARGS, step },
      global: globalProvide,
    });
    expect(wrapper.find(".arg-input").exists()).toBe(false);
  });

  it("calls updateQueryStep with new value on input change", async () => {
    const step = makeStep({
      args: [[{ selectedOption: "signed", value: "" }]],
    });
    const wrapper = mount(QueryArgument, {
      props: { arguments: SINGLE_ARGS, step },
      global: globalProvide,
    });
    vi.clearAllMocks();
    const input = wrapper.find(".arg-input");
    await input.setValue("99");
    expect(updateQueryStep).toHaveBeenCalledOnce();
    const committed = updateQueryStep.mock.calls[0]?.[2];
    expect(committed.args[0][0].value).toBe("99");
  });

  describe("repeatable arguments", () => {
    it("shows add-entry button when repeatable", () => {
      const step = makeStep({
        type: "values",
        args: [
          [
            { selectedOption: "string", value: "a" },
            { selectedOption: "unsigned", value: "1" },
          ],
        ],
      });
      const wrapper = mount(QueryArgument, {
        props: { arguments: REPEATABLE_ARGS, step },
        global: globalProvide,
      });
      expect(wrapper.find(".arg-add-entry").exists()).toBe(true);
    });

    it("does not show add-entry button when not repeatable", () => {
      const step = makeStep({
        args: [[{ selectedOption: "signed", value: "1" }]],
      });
      const wrapper = mount(QueryArgument, {
        props: { arguments: SINGLE_ARGS, step },
        global: globalProvide,
      });
      expect(wrapper.find(".arg-add-entry").exists()).toBe(false);
    });

    it("adds a new empty entry on + click", async () => {
      const step = makeStep({
        type: "values",
        args: [
          [
            { selectedOption: "string", value: "a" },
            { selectedOption: "unsigned", value: "1" },
          ],
        ],
      });
      const wrapper = mount(QueryArgument, {
        props: { arguments: REPEATABLE_ARGS, step },
        global: globalProvide,
      });
      vi.clearAllMocks();
      await wrapper.find(".arg-add-entry").trigger("click");
      expect(updateQueryStep).toHaveBeenCalledOnce();
      const committed = updateQueryStep.mock.calls[0]?.[2];
      expect(committed.args).toHaveLength(2);
    });

    it("shows remove-entry button only when more than one entry exists", () => {
      const step = makeStep({
        type: "values",
        args: [
          [
            { selectedOption: "string", value: "a" },
            { selectedOption: "unsigned", value: "1" },
          ],
          [
            { selectedOption: "string", value: "b" },
            { selectedOption: "unsigned", value: "2" },
          ],
        ],
      });
      const wrapper = mount(QueryArgument, {
        props: { arguments: REPEATABLE_ARGS, step },
        global: globalProvide,
      });
      expect(wrapper.findAll(".arg-remove-entry")).toHaveLength(2);
    });

    it("does not show remove-entry button when only one entry exists", () => {
      const step = makeStep({
        type: "values",
        args: [
          [
            { selectedOption: "string", value: "a" },
            { selectedOption: "unsigned", value: "1" },
          ],
        ],
      });
      const wrapper = mount(QueryArgument, {
        props: { arguments: REPEATABLE_ARGS, step },
        global: globalProvide,
      });
      expect(wrapper.find(".arg-remove-entry").exists()).toBe(false);
    });

    it("removes an entry on − click", async () => {
      const step = makeStep({
        type: "values",
        args: [
          [
            { selectedOption: "string", value: "a" },
            { selectedOption: "unsigned", value: "1" },
          ],
          [
            { selectedOption: "string", value: "b" },
            { selectedOption: "unsigned", value: "2" },
          ],
        ],
      });
      const wrapper = mount(QueryArgument, {
        props: { arguments: REPEATABLE_ARGS, step },
        global: globalProvide,
      });
      vi.clearAllMocks();
      await wrapper.find(".arg-remove-entry").trigger("click");
      expect(updateQueryStep).toHaveBeenCalledOnce();
      const committed = updateQueryStep.mock.calls[0]?.[2];
      expect(committed.args).toHaveLength(1);
    });
  });

  it("calls updateQueryStep when dropdown selection changes", async () => {
    const step = makeStep({
      args: [[{ selectedOption: "signed", value: "5" }]],
    });
    const wrapper = mount(QueryArgument, {
      props: { arguments: SINGLE_ARGS, step },
      global: globalProvide,
    });
    vi.clearAllMocks();
    const dropdown = wrapper.findComponent({ name: "QueryArgumentDropdown" });
    await dropdown.vm.$emit("update:modelValue", "string");
    expect(updateQueryStep).toHaveBeenCalledOnce();
    const committed = updateQueryStep.mock.calls[0]?.[2];
    expect(committed.args[0][0].selectedOption).toBe("string");
    expect(committed.args[0][0].value).toBeUndefined();
  });

  it("does not commit when tab is missing", () => {
    mount(QueryArgument, {
      props: { arguments: SINGLE_ARGS, step: makeStep() },
      global: {
        provide: {
          queryId: ref("q1"),
          activeTab: ref(undefined),
        },
      },
    });
    expect(updateQueryStep).not.toHaveBeenCalled();
  });

  it("focuses the first dropdown when autoFocus is true", async () => {
    const step = makeStep({
      args: [[{ selectedOption: "signed", value: "1" }]],
    });
    const wrapper = mount(QueryArgument, {
      props: { arguments: SINGLE_ARGS, step, autoFocus: true },
      global: globalProvide,
      attachTo: document.body,
    });
    await wrapper.vm.$nextTick();
    await wrapper.vm.$nextTick();
    expect(document.activeElement?.tagName).toBe("BUTTON");
    wrapper.unmount();
  });
});
