import { ref, type Reactive } from "vue";

const inputs = ref(new Map<Symbol, Map<string, Reactive<Input>>>());

const getContentInputs = (
    contentKey: Symbol,
): Map<string, Reactive<Input>> | undefined => {
    return inputs.value.get(contentKey);
};

const getInputValue = <T = string | number | boolean | undefined>(
    contentKey: Symbol,
    inputKey: string,
): T => {
    return inputs.value.get(contentKey)?.get(inputKey)?.value as T;
};

const setInputValue = (
    contentKey: Symbol,
    inputKey: string | undefined,
    value: string | number | boolean | undefined,
): void => {
    if (!inputKey || !inputKey.length) {
        return;
    }
    const input = inputs.value.get(contentKey)?.get(inputKey);
    if (!input) {
        return;
    }
    input.error = undefined;
    input.value = value;
};

const clearInputs = (contentKey: Symbol) => {
    const inputsMap = inputs.value.get(contentKey);
    inputsMap?.clear();
};

const addInput = (
    contentKey: Symbol,
    inputKey: string,
    params: Input,
): void => {
    const inputsMap = inputs.value.get(contentKey);
    if (!inputsMap) {
        inputs.value.set(contentKey, new Map());
    }
    inputs.value.get(contentKey)?.set(inputKey, params);
};

const clearAllInputs = (): void => {
    inputs.value.clear();
};

const checkInputsRules = (contentKey: Symbol): boolean => {
    const inputsMap = inputs.value.get(contentKey);
    if (!inputsMap) {
        return true;
    }
    let isValid = true;
    inputsMap.forEach((input) => {
        if (input.required && !input.value) {
            input.error = "This field is required";
            isValid = false;
        }
    });
    return isValid;
};

export const useContentInputs = () => {
    return {
        getContentInputs,
        clearInputs,
        addInput,
        getInputValue,
        setInputValue,
        clearAllInputs,
        checkInputsRules,
    };
};
