import { ref, type Ref } from "vue";

const inputs = ref(
    new Map<Symbol, Map<string, Ref<string | number | boolean | undefined>>>(),
);

const getContentInputs = (contentKey: Symbol) => {
    return inputs.value.get(contentKey);
};

const getInputValue = (contentKey: Symbol, inputKey: string) => {
    return inputs.value.get(contentKey)?.get(inputKey);
};

const clearInputs = (contentKey: Symbol) => {
    const inputsMap = inputs.value.get(contentKey);
    inputsMap?.clear();
};

const addInput = (
    contentKey: Symbol,
    inputKey: string,
    value: Ref<string | number | boolean | undefined>,
) => {
    const inputsMap = inputs.value.get(contentKey);
    if (!inputsMap) {
        inputs.value.set(contentKey, new Map());
    }
    inputs.value.get(contentKey)?.set(inputKey, value);
};

export const useContentInputs = () => {
    return { getContentInputs, clearInputs, addInput, getInputValue };
};
