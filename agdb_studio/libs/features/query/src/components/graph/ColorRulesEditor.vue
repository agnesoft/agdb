<script lang="ts" setup>
import { computed, ref, watch } from "vue";
import {
  ruleScale,
  unusedScale,
  type ColorRule,
  type PaletteSlot,
} from "../../utils/graphColorRules";
import ColorSwatchPicker from "./ColorSwatchPicker.vue";
import RampPicker from "./RampPicker.vue";
import type { RampName } from "../../utils/graphStyle";

/** A key of the shown elements with its values, most common first. */
export type ValueIndexEntry = {
  key: string;
  /** Every value reads as a number, so the key can be a gradient. */
  numeric?: boolean;
  values: { value: string; count: number }[];
};

/**
 * Colours elements by a property: each value its own colour, a gradient by a
 * numeric value, or one colour for elements that have the key, or a given
 * value of it. Keys and values are picked from the shown elements. Distance
 * rules are added elsewhere and listed here. The first matching rule colours
 * an element.
 */
const props = withDefaults(
  defineProps<{
    rules: readonly ColorRule[];
    palette: readonly string[];
    ramps: Readonly<Record<RampName, readonly string[]>>;
    keys: readonly ValueIndexEntry[];
    /** The label of a distance rule's start node. */
    elementLabel?: (id: number) => string;
  }>(),
  { elementLabel: (id: number) => `element ${id}` },
);

const emit = defineEmits<{ "update:rules": [rules: ColorRule[]] }>();

/** Values offered per key; the rest of a long tail is left out. */
const MAX_VALUES = 50;

const EACH = "__each__";
const GRADIENT = "__gradient__";
const ANY = "__any__";

const key = ref("");
const which = ref(EACH);
const color = ref<PaletteSlot>(0);

const keyEntry = computed(() =>
  props.keys.find((entry) => entry.key === key.value),
);
const values = computed(
  () => keyEntry.value?.values.slice(0, MAX_VALUES) ?? [],
);
/** Rules that pick one colour, rather than many from the palette or the gradient. */
const singleColor = computed(
  () => which.value !== EACH && which.value !== GRADIENT,
);

// A new key starts with one colour per value, the usual first look.
watch(key, () => {
  which.value = EACH;
});
// New results can make the key no longer numeric, and the gradient unavailable.
watch(
  () => keyEntry.value?.numeric,
  (numeric) => {
    if (!numeric && which.value === GRADIENT) which.value = EACH;
  },
);

const ruleLabel = (rule: ColorRule): string => {
  if (rule.kind === "palette") return `${rule.key}: each value its own colour`;
  if (rule.kind === "gradient") return `${rule.key}: gradient`;
  if (rule.kind === "distance") {
    return rule.from === "start"
      ? "distance from the search start"
      : `distance from ${props.elementLabel(rule.from)}`;
  }
  return rule.op === "exists"
    ? `${rule.key}: any value`
    : `${rule.key} = ${rule.value ?? ""}`;
};

const addRule = () => {
  /* v8 ignore if -- @preserve: the button is disabled without a key */
  if (!key.value) return;
  const id = crypto.randomUUID();
  const rule: ColorRule =
    which.value === EACH
      ? { kind: "palette", id, key: key.value }
      : which.value === GRADIENT
        ? {
            kind: "gradient",
            id,
            key: key.value,
            // Its own scale, apart from the other gradients.
            scale: unusedScale(props.rules),
          }
        : which.value === ANY
          ? {
              kind: "property",
              id,
              key: key.value,
              op: "exists",
              color: color.value,
            }
          : {
              kind: "property",
              id,
              key: key.value,
              op: "eq",
              value: which.value,
              color: color.value,
            };
  emit("update:rules", [...props.rules, rule]);
  // The next rule gets the next colour, so rules do not share one by default.
  color.value = (color.value + 1) % Math.max(1, props.palette.length);
};

const removeRule = (id: string) => {
  emit(
    "update:rules",
    props.rules.filter((rule) => rule.id !== id),
  );
};

const setRuleColor = (id: string, slot: PaletteSlot | undefined) => {
  /* v8 ignore if -- @preserve: rule colours cannot be cleared */
  if (slot === undefined) return;
  emit(
    "update:rules",
    props.rules.map((rule) =>
      rule.id === id && rule.kind === "property"
        ? { ...rule, color: slot }
        : rule,
    ),
  );
};

const setRuleScale = (id: string, scale: RampName) => {
  emit(
    "update:rules",
    props.rules.map((rule) =>
      rule.id === id && (rule.kind === "gradient" || rule.kind === "distance")
        ? { ...rule, scale }
        : rule,
    ),
  );
};

/** Moves a rule up, so it takes precedence over the one before it. */
const raiseRule = (index: number) => {
  const rules = [...props.rules];
  const [rule] = rules.splice(index, 1);
  /* v8 ignore if -- @preserve: only rules after the first can move up */
  if (!rule || index <= 0) return;
  rules.splice(index - 1, 0, rule);
  emit("update:rules", rules);
};
</script>

<template>
  <div class="color-rules-editor" data-testid="color-rules-editor">
    <ol v-if="rules.length" class="rules">
      <li
        v-for="(rule, index) in rules"
        :key="rule.id"
        class="rule"
        data-testid="color-rule"
      >
        <ColorSwatchPicker
          v-if="rule.kind === 'property'"
          :palette="palette"
          :model-value="rule.color"
          :clearable="false"
          label="Rule colour"
          @update:model-value="(slot) => setRuleColor(rule.id, slot)"
        />
        <RampPicker
          v-else-if="rule.kind !== 'palette'"
          :ramps="ramps"
          :model-value="ruleScale(rule)"
          label="Gradient colours"
          @update:model-value="(scale) => setRuleScale(rule.id, scale)"
        />
        <span
          v-else
          class="palette-preview"
          title="Each value its own colour, see the legend"
          aria-hidden="true"
        >
          <span
            v-for="(c, i) in palette"
            :key="i"
            :style="{ backgroundColor: c }"
          ></span>
        </span>
        <span class="rule-label">{{ ruleLabel(rule) }}</span>
        <button
          v-if="index > 0"
          type="button"
          class="button icon-button"
          title="Apply before the rule above"
          aria-label="Move up"
          data-testid="color-rule-up"
          @click="raiseRule(index)"
        >
          ↑
        </button>
        <button
          type="button"
          class="button icon-button"
          title="Remove rule"
          aria-label="Remove rule"
          data-testid="color-rule-remove"
          @click="removeRule(rule.id)"
        >
          ×
        </button>
      </li>
    </ol>

    <p v-if="!keys.length" class="empty" data-testid="color-rule-no-keys">
      The results have no properties to colour by.
    </p>
    <form v-else class="new-rule" @submit.prevent="addRule">
      <select v-model="key" aria-label="Property" data-testid="color-rule-key">
        <option value="" disabled>Property…</option>
        <option v-for="entry in keys" :key="entry.key" :value="entry.key">
          {{ entry.key }}
        </option>
      </select>
      <select
        v-model="which"
        aria-label="Which values"
        :disabled="!key"
        data-testid="color-rule-which"
      >
        <option :value="EACH">a colour per value</option>
        <option v-if="keyEntry?.numeric" :value="GRADIENT">a gradient</option>
        <option :value="ANY">any value</option>
        <option v-for="v in values" :key="v.value" :value="v.value">
          = {{ v.value }} ({{ v.count }})
        </option>
      </select>
      <ColorSwatchPicker
        v-if="singleColor"
        v-model="color"
        :palette="palette"
        :clearable="false"
        label="Colour of the new rule"
      />
      <button
        type="submit"
        class="button"
        :disabled="!key"
        data-testid="color-rule-add"
      >
        Add
      </button>
    </form>
  </div>
</template>

<style lang="less" scoped>
.color-rules-editor {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}
.icon-button {
  padding: 0 0.4rem;
  line-height: 1.3rem;
}
.rules {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}
.rule {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}
.rule-label {
  margin-right: auto;
  word-break: break-word;
}
.palette-preview {
  display: inline-flex;
  flex: 0 0 auto;
  overflow: hidden;
  border-radius: 0.2rem;
  span {
    width: 0.3rem;
    height: 0.8rem;
  }
}
.new-rule {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.3rem;
  select {
    flex: 1 1 7rem;
    min-width: 0;
  }
}
.empty {
  color: var(--color-text-muted);
}
</style>
