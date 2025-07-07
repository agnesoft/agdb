import { describe, beforeEach, vi, it, expect } from "vitest";
import AgdbContent from "./AgdbContent.vue";
import { mount } from "@vue/test-utils";
import { useContentInputs } from "../../composables/content/inputs";
import type { Input } from "@/composables/content/types";
import type { AsyncComponent } from "@/types/asyncComponents";

const { addInput, getInputValue, clearAllInputs, checkInputsRules } =
  useContentInputs();

const testInput: Input = {
  key: "testKey",
  label: "Test label",
  type: "text",
  autofocus: true,
  required: true,
  value: "Test value",
};

describe("AgdbContent", () => {
  const testContentKey = Symbol("test");
  beforeEach(() => {
    vi.clearAllMocks();
    clearAllInputs();
  });

  it("renders the content", () => {
    const wrapper = mount(AgdbContent, {
      props: {
        content: [
          {
            paragraph: [
              {
                text: "Test Body",
              },
            ],
          },
          {
            component: "my-component" as unknown as AsyncComponent,
          },
          {
            input: testInput,
          },
        ],
        contentKey: testContentKey,
      },
    });
    expect(wrapper.html()).toContain("Test Body");
    expect(wrapper.html()).toContain("my-component");
    expect(wrapper.html()).toContain("Test label");
  });
  it("change the input value on user input", async () => {
    addInput(testContentKey, testInput);
    const wrapper = mount(AgdbContent, {
      props: {
        content: [
          {
            input: testInput,
          },
        ],
        contentKey: testContentKey,
      },
    });
    const input = wrapper.find("input");
    expect(getInputValue(testContentKey, testInput.key)).toBe("Test value");
    expect(input.element.value).toBe("Test value");
    input.element.value = "test value 2";
    await input.trigger("input");
    expect(getInputValue(testContentKey, testInput.key)).toBe("test value 2");
  });

  it("sets focus on the input with autofocus", async () => {
    addInput(testContentKey, testInput);
    const wrapper = mount(AgdbContent, {
      props: {
        content: [
          {
            input: testInput,
          },
        ],
        contentKey: testContentKey,
      },
      attachTo: document.body,
    });
    await wrapper.vm.$nextTick();
    const input = wrapper.find("input");
    expect(input.element.matches(":focus")).toBe(true);
  });

  it("should not set focus on the input without autofocus", async () => {
    const inputWithoutFocus: Input = {
      key: "testKey",
      label: "Test label",
      type: "text",
      required: true,
      value: "Test value",
    };
    addInput(testContentKey, inputWithoutFocus);
    const wrapper = mount(AgdbContent, {
      props: {
        content: [
          {
            input: inputWithoutFocus,
          },
        ],
        contentKey: testContentKey,
      },
      attachTo: document.body,
    });
    await wrapper.vm.$nextTick();
    const input = wrapper.find("input");
    expect(input.element.matches(":focus")).toBe(false);
  });

  it("should render select input and change value", async () => {
    const selectInput: Input = {
      key: "testKey",
      label: "Test input",
      type: "select",
      options: [
        { value: "test-option", label: "Test" },
        { value: "test-option-2", label: "Test2" },
      ],
      value: "test-option",
    };
    addInput(testContentKey, selectInput);
    const wrapper = mount(AgdbContent, {
      props: {
        content: [
          {
            input: selectInput,
          },
        ],
        contentKey: testContentKey,
      },
    });
    const select = wrapper.find("select");
    expect(select.element.value).toBe("test-option");
    expect(getInputValue(testContentKey, selectInput.key)).toBe("test-option");
    select.element.value = "test-option-2";
    await select.trigger("change");
    expect(getInputValue(testContentKey, selectInput.key)).toBe(
      "test-option-2",
    );
  });

  it("should display error message when input rules are false", async () => {
    const requiredInput: Input = {
      key: "testKey",
      label: "Test input",
      type: "text",
      required: true,
      value: "",
      rules: [() => "required"],
    };
    addInput(testContentKey, requiredInput);
    const wrapper = mount(AgdbContent, {
      props: {
        content: [
          {
            input: requiredInput,
          },
        ],
        contentKey: testContentKey,
      },
    });
    checkInputsRules(testContentKey);
    await wrapper.vm.$nextTick();
    expect(wrapper.text()).toContain("required");
  });
});
