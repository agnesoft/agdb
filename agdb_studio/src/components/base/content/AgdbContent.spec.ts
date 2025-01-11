import { describe, beforeEach, vi, it, expect } from "vitest";
import AgdbContent from "./AgdbContent.vue";
import { mount } from "@vue/test-utils";
import { useContentInputs } from "@/composables/content/inputs";
import { ref } from "vue";

const { addInput, getInputValue, clearAllInputs } = useContentInputs();

describe("AgdbContent", () => {
    const testKey = Symbol("test");
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
                        input: {
                            key: "test",
                            type: "text",
                            label: "Test input",
                        },
                    },
                ],
                contentKey: testKey,
            },
        });
        expect(wrapper.html()).toContain("Test Body");
        expect(wrapper.html()).toContain("my-component");
        expect(wrapper.html()).toContain("Test input");
    });
    it("change the input value on user input", async () => {
        const inputValue = ref("");
        addInput(testKey, "test", inputValue);
        const wrapper = mount(AgdbContent, {
            props: {
                content: [
                    {
                        input: {
                            key: "test",
                            type: "text",
                            label: "Test input",
                        },
                    },
                ],
                contentKey: testKey,
            },
        });
        const input = wrapper.find("input");
        expect(getInputValue(testKey, "test")).toBe("");
        expect(input.element.value).toBe("");
        input.element.value = "test value";
        await input.trigger("input");
        expect(getInputValue(testKey, "test")).toBe("test value");
    });
    it("sets focus on the input with autofocus", async () => {
        const inputValue = ref("");
        addInput(testKey, "test", inputValue);
        const wrapper = mount(AgdbContent, {
            props: {
                content: [
                    {
                        input: {
                            key: "test",
                            type: "text",
                            label: "Test input",
                            autofocus: true,
                        },
                    },
                ],
                contentKey: testKey,
            },
            attachTo: document.body,
        });
        await wrapper.vm.$nextTick();
        const input = wrapper.find("input");
        expect(input.element.matches(":focus")).toBe(true);
    });
    it("should render select input and change value", async () => {
        const inputValue = ref("test");
        addInput(testKey, "test", inputValue);
        const wrapper = mount(AgdbContent, {
            props: {
                content: [
                    {
                        input: {
                            key: "test",
                            type: "select",
                            label: "Test input",
                            options: [
                                { value: "test", label: "Test" },
                                { value: "test2", label: "Test2" },
                            ],
                        },
                    },
                ],
                contentKey: testKey,
            },
        });
        const select = wrapper.find("select");
        expect(select.element.value).toBe("test");
        expect(getInputValue(testKey, "test")).toBe("test");
        select.element.value = "test2";
        await select.trigger("change");
        expect(getInputValue(testKey, "test")).toBe("test2");
    });
});
