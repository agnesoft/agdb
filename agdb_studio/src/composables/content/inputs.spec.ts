import { describe, it, expect, beforeEach } from "vitest";
import { useContentInputs } from "@/composables/content/inputs";

const {
    addInput,
    getInputValue,
    setInputValue,
    getContentInputs,
    clearAllInputs,
    clearInputs,
    checkInputsRules,
} = useContentInputs();

const testInput: Input = {
    key: "testKey",
    label: "Test label",
    type: "text",
    autofocus: true,
    required: true,
    value: "Test value",
};

const testInput2: Input = {
    key: "testKey2",
    label: "Test label 2",
    type: "text",
    autofocus: true,
    required: true,
    value: "Test value 2",
};

describe("Content inputs", () => {
    const contentKey = Symbol();
    beforeEach(() => {
        clearAllInputs();
    });
    it("adds an input", () => {
        addInput(contentKey, testInput);
        const inputs = getContentInputs(contentKey);
        expect(inputs).toBeDefined();
        expect(inputs?.get(testInput.key)?.value).toBe("Test value");
    });
    it("gets an input value", () => {
        addInput(contentKey, testInput);
        expect(getInputValue(contentKey, testInput.key)).toBe("Test value");
    });
    it("sets an input value", () => {
        addInput(contentKey, testInput);
        const newValue = "new value";
        const newValue2 = "new value 2";
        setInputValue(contentKey, testInput.key, newValue);

        addInput(contentKey, testInput2);
        setInputValue(contentKey, testInput2.key, newValue2);
        expect(getInputValue(contentKey, testInput.key)).toBe(newValue);
        expect(getInputValue(contentKey, testInput2.key)).toBe(newValue2);
    });
    it("clears all inputs", () => {
        addInput(contentKey, testInput);
        clearAllInputs();
        const inputs = getContentInputs(contentKey);
        expect(inputs).toBeUndefined();
    });
    it("clears inputs", () => {
        addInput(contentKey, testInput);
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

    it("checks input rules", () => {
        addInput(contentKey, testInput);
        setInputValue(contentKey, testInput.key, "bla");
        expect(checkInputsRules(contentKey)).toBe(true);
    });

    it("checks input rules with required input", () => {
        addInput(contentKey, testInput);
        setInputValue(contentKey, testInput.key, "");
        expect(checkInputsRules(contentKey)).toBe(false);
    });

    it("checks input rules with custom rule", () => {
        addInput(contentKey, {
            ...testInput,
            rules: [
                (value: string) => (value === "test" ? "error" : undefined),
            ],
        });
        setInputValue(contentKey, testInput.key, "test");
        expect(checkInputsRules(contentKey)).toBe(false);
    });

    it("checks input rules with multiple inputs", () => {
        addInput(contentKey, testInput);
        addInput(contentKey, testInput2);
        setInputValue(contentKey, testInput.key, "");
        setInputValue(contentKey, testInput2.key, "");
        expect(checkInputsRules(contentKey)).toBe(false);
    });

    it("checks input rules with multiple inputs and custom rule", () => {
        addInput(contentKey, {
            ...testInput,
            rules: [
                (value: string) => (value === "test" ? "error" : undefined),
            ],
        });
        addInput(contentKey, testInput2);
        setInputValue(contentKey, testInput.key, "test");
        setInputValue(contentKey, testInput2.key, "");
        expect(checkInputsRules(contentKey)).toBe(false);
    });

    it("checks input rules with multiple inputs and custom rule", () => {
        addInput(contentKey, {
            ...testInput,
            rules: [
                (value: string) => (value === "test" ? "error" : undefined),
            ],
        });
        addInput(contentKey, testInput2);
        setInputValue(contentKey, testInput.key, "test 2");
        setInputValue(contentKey, testInput2.key, "test value 2");
        expect(checkInputsRules(contentKey)).toBe(true);
    });

    it("handles undefined inputs", () => {
        expect(checkInputsRules(contentKey)).toBe(true);
    });
});
