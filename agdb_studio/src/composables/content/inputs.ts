import { ref, type Ref } from "vue";

const inputs = ref(
    new Map<Symbol, Map<string, Ref<string | number | boolean | undefined>>>(),
);

const getContentInputs = (
    contentKey: Symbol,
): Map<string, Ref<string | number | boolean | undefined>> | undefined => {
    return inputs.value.get(contentKey);
};

const getInputValue = (
    contentKey: Symbol,
    inputKey: string,
): string | number | boolean | undefined => {
    console.log(inputs.value.get(contentKey));
    return inputs.value.get(contentKey)?.get(inputKey)?.value;
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

    input.value = value;
};

const clearInputs = (contentKey: Symbol) => {
    const inputsMap = inputs.value.get(contentKey);
    inputsMap?.clear();
};

const addInput = (
    contentKey: Symbol,
    inputKey: string,
    value: Ref<string | number | boolean | undefined>,
): void => {
    const inputsMap = inputs.value.get(contentKey);
    if (!inputsMap) {
        inputs.value.set(contentKey, new Map());
    }
    inputs.value.get(contentKey)?.set(inputKey, value);
};

const clearAllInputs = (): void => {
    inputs.value.clear();
};

export const useContentInputs = () => {
    return {
        getContentInputs,
        clearInputs,
        addInput,
        getInputValue,
        setInputValue,
        clearAllInputs,
    };
};
