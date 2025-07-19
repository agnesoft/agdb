<script lang="ts" setup>
import { onMounted, ref, type PropType } from "vue";
import { useContentInputs } from "../../composables/content/inputs";
import FadeTransition from "@agdb-studio/design/src/components/transitions/FadeTransition.vue";
import type { Content } from "../../composables/content/types";

const props = defineProps({
  content: { type: Array as PropType<Content[]>, required: true },
  contentKey: { type: Symbol, required: true },
});

const { getContentInputs, setInputValue, getInputValue } = useContentInputs();
const inputs = getContentInputs(props.contentKey) ?? new Map();

const autofocusElement = ref();

onMounted(() => {
  autofocusElement.value?.focus();
});

const getInputKey = (key: string) => {
  return `${props.contentKey.toString()}-${key}`;
};
</script>

<template>
  <div class="agdb-content">
    <div v-for="(part, index) in content" :key="index">
      <p v-if="part.paragraph?.length">
        <span
          v-for="(text, index2) in part.paragraph"
          :key="index2"
          :style="text.style"
          :class="text.className"
        >
          {{ text.text }}
        </span>
      </p>
      <div v-if="part.input && part.input.type === 'checkbox'">
        <input
          :id="getInputKey(part.input.key)"
          class="checkbox"
          :type="part.input.type"
          :checked="getInputValue(props.contentKey, part.input.key)"
          @change="
            (event: Event) => {
              setInputValue(
                props.contentKey,
                part.input?.key,
                (event.target as HTMLInputElement).checked,
              );
            }
          "
        />
        <label :for="getInputKey(part.input.key)">{{ part.input.label }}</label>
      </div>
      <div v-else-if="part.input" class="input-row">
        <label :for="getInputKey(part.input.key)">{{ part.input.label }}</label>
        <div :class="{ 'error-input': part.input.error }">
          <select
            v-if="
              inputs.get(part.input.key) !== undefined &&
              part.input.type === 'select'
            "
            :id="getInputKey(part.input.key)"
            :value="getInputValue(props.contentKey, part.input.key)"
            @change="
              (event: Event) => {
                setInputValue(
                  props.contentKey,
                  part.input?.key,
                  (event.target as HTMLSelectElement).value,
                );
              }
            "
          >
            <option
              v-for="option in part.input.options"
              :key="option.value"
              :value="option.value"
            >
              {{ option.label }}
            </option>
          </select>
          <input
            v-else-if="inputs.get(part.input.key) !== undefined"
            :id="getInputKey(part.input.key)"
            :ref="
              (el) => {
                if (part.input?.autofocus) autofocusElement = el;
              }
            "
            :type="part.input.type"
            :value="getInputValue(props.contentKey, part.input.key)"
            @input="
              (event: Event) => {
                setInputValue(
                  props.contentKey,
                  part.input?.key,
                  (event.target as HTMLInputElement).value,
                );
              }
            "
          />
          <FadeTransition>
            <div v-if="part.input.error" class="error-message">
              {{ part.input.error }}
            </div>
          </FadeTransition>
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="less" scoped>
.agdb-content {
  p {
    margin-bottom: 1rem;
  }
}
.input-row {
  display: grid;
  grid-template-columns: minmax(60px, 170px) minmax(150px, 1fr);
  grid-gap: 1rem;
  margin-bottom: 1rem;
  position: relative;

  label {
    justify-self: end;
    align-self: center;
  }

  input,
  select {
    width: 100%;
    padding: 0.2rem;
    border: 1px solid #ccc;
    border-radius: 5px;
    transition: outline 0.3s ease;
  }
  .error-input {
    input,
    select {
      outline: 2px solid var(--red);
    }
  }
}
.error-message {
  font-size: 0.8rem;
  position: absolute;
  bottom: -0.5rem;
  right: 0;
  background-color: var(--red-2);
  color: var(--white);
  border: 1px solid var(--red);
  padding: 0.1rem 0.5rem;
  border-radius: 5px;
  z-index: 1;
  max-width: 40%;
}

.checkbox {
  margin-right: 0.5rem;
}

@media (max-width: 768px) {
  .input-row {
    grid-template-columns: 1fr;
  }
  .error-message {
    max-width: 60%;
  }
}
</style>
