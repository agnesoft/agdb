<script lang="ts" setup>
import { onMounted, ref, type PropType } from "vue";
import { useContentInputs } from "@/composables/content/inputs";

const props = defineProps({
    content: { type: Array as PropType<Content[]>, required: true },
    contentKey: { type: Symbol, required: true },
});

const { getContentInputs, setInputValue } = useContentInputs();
const inputs = getContentInputs(props.contentKey) ?? new Map();

const autofocusElement = ref();

onMounted(() => {
    autofocusElement.value?.focus();
});
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
            <div v-if="part.component">
                <component :is="part.component" />
            </div>
            <div v-if="part.input" class="input-row">
                <label>{{ part.input.label }}</label>
                <select
                    v-if="
                        inputs.get(part.input.key) !== undefined &&
                        part.input.type === 'select'
                    "
                >
                    <option
                        v-for="(option, index) in part.input.options"
                        :key="index"
                        :value="option.value"
                    >
                        {{ option.label }}
                    </option>
                </select>
                <input
                    v-else-if="inputs.get(part.input.key) !== undefined"
                    :type="part.input.type"
                    :ref="
                        (el) => {
                            if (part.input?.autofocus) autofocusElement = el;
                        }
                    "
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
    grid-template-columns: minmax(60px, 150px) minmax(150px, 1fr);
    grid-gap: 1rem;
    margin-bottom: 1rem;
}
</style>
