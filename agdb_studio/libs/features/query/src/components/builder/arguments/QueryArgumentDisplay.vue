<script lang="ts" setup>
import { computed } from "vue";
import {
  OPTION_SHORTCUT_MAP,
  OPTION_TYPE_MAP,
  type QueryArguments,
} from "../../../mock/queryApiMock";
import type { QueryStep } from "../../../composables/types";

const props = defineProps<{
  arguments: QueryArguments;
  step: QueryStep;
}>();

const display = computed(() => {
  const entries = props.step.args;
  if (!entries?.length) return null;

  return entries
    .map((entry) => {
      const parts = entry
        .flatMap((fv) => {
          const optionLabel =
            OPTION_SHORTCUT_MAP[fv.selectedOption] ?? fv.selectedOption;
          const hasValue = OPTION_TYPE_MAP[fv.selectedOption] != null;
          const hasLiteralValue =
            fv.value !== undefined && fv.value !== null && fv.value !== "";
          return hasValue && hasLiteralValue
            ? [optionLabel, fv.value]
            : [optionLabel];
        })
        .join(", ");
      return `(${parts})`;
    })
    .join(", ");
});
</script>

<template>
  <span class="arg-display" :class="{ placeholder: !display }">{{
    display ?? "(…)"
  }}</span>
</template>

<style lang="less" scoped>
.arg-display {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  font-style: italic;
  white-space: nowrap;

  &.placeholder {
    opacity: 0.45;
  }
}
</style>
