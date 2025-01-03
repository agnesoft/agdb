import { describe, it, expect, beforeEach } from "vitest";
import { useContentInputs } from "@/composables/content/inputs";
import { ref } from "vue";

const {
    addInput,
    getInputValue,
    setInputValue,
    getContentInputs,
    clearAllInputs,
    clearInputs,
} = useContentInputs();

describe("Content inputs", () => {
    const contentKey = Symbol();
    beforeEach(() => {
        clearAllInputs();
    });
    it("adds an input", () => {
        const inputKey = "test";
        const value = ref("test value");
        addInput(contentKey, inputKey, value);
        const inputs = getContentInputs(contentKey);
        expect(inputs).toBeDefined();
        expect(inputs?.get(inputKey)?.value).toBe(value.value);
    });
    it("gets an input value", () => {
        const inputKey = "test";
        const value = ref("test value");
        addInput(contentKey, inputKey, value);
        expect(getInputValue(contentKey, inputKey)).toBe(value.value);
    });
    it("sets an input value", () => {
        const inputKey = "test";
        const value = ref("test value");
        addInput(contentKey, inputKey, value);
        const newValue = "new value";
        const newValue2 = "new value 2";
        setInputValue(contentKey, inputKey, newValue);

        addInput(contentKey, "test2", ref("test value 2"));
        setInputValue(contentKey, "test2", newValue2);
        expect(getInputValue(contentKey, inputKey)).toBe(newValue);
        expect(getInputValue(contentKey, "test2")).toBe(newValue2);
    });
    it("clears all inputs", () => {
        const inputKey = "test";
        const value = ref("test value");
        addInput(contentKey, inputKey, value);
        clearAllInputs();
        const inputs = getContentInputs(contentKey);
        expect(inputs).toBeUndefined();
    });
    it("clears inputs", () => {
        const inputKey = "test";
        const value = ref("test value");
        addInput(contentKey, inputKey, value);
        clearInputs(contentKey);
        const inputs = getContentInputs(contentKey);
        expect(inputs?.size).toBe(0);
    });
    it("does not set value if input does not exist", () => {
        setInputValue(contentKey, "test", "test");
        expect(getInputValue(contentKey, "test")).toBeUndefined();
    });
    it("does not set value if input key is empty", () => {
        setInputValue(contentKey, "", "test");
        expect(getInputValue(contentKey, "")).toBeUndefined();
    });
    it("does not set value if input key is undefined", () => {
        setInputValue(contentKey, undefined, "test");
        expect(getInputValue(contentKey, "")).toBeUndefined();
    });
});
