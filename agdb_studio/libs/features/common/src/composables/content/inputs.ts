import { reactive, ref, type Reactive } from "vue";

const inputs = ref(new Map<symbol, Map<string, Reactive<Input>>>());

const getContentInputs = (
  contentKey: symbol,
): Map<string, Reactive<Input>> | undefined => {
  return inputs.value.get(contentKey);
};

const getInputValue = <T = string | number | boolean | undefined>(
  contentKey: symbol,
  inputKey: string,
): T => {
  return inputs.value.get(contentKey)?.get(inputKey)?.value as T;
};

const setInputValue = (
  contentKey: symbol,
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

const clearInputs = (contentKey: symbol) => {
  const inputsMap = inputs.value.get(contentKey);
  inputsMap?.forEach((input) => {
    input.error = undefined;
    input.value = undefined;
  });
  inputsMap?.clear();
};

const addInput = (contentKey: symbol, params: Input): void => {
  const inputsMap = inputs.value.get(contentKey);
  if (!inputsMap) {
    inputs.value.set(contentKey, new Map());
  }
  inputs.value.get(contentKey)?.set(params.key, reactive(params));
};

const clearAllInputs = (): void => {
  inputs.value.clear();
};

const checkInputsRules = (contentKey: symbol): boolean => {
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
    if (input.rules) {
      input.rules.forEach((rule) => {
        const error = rule(input.value as string);
        if (error) {
          input.error = error;
          isValid = false;
        }
      });
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
