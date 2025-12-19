<script lang="ts" setup>
import { computed, ref } from "vue";
import { queryApiMock } from "../../mock/queryApiMock";
import type { QueryStep, QueryType } from "../../composables/types";
import QueryHinter from "./QueryHinter.vue";
import FadeTransition from "@agdb-studio/design/src/components/transitions/FadeTransition.vue";
import { vOnClickOutside } from "@vueuse/components";

const props = defineProps<{
  prevStep?: QueryStep;
}>();

const content = ref("");
const contentInput = ref<HTMLElement | null>(null);

const updateContent = () => {
  /* v8 ignore next -- @preserve */
  content.value =
    contentInput.value?.innerText.trim().replace(/[^\w]/g, "") ?? "";
};

const onKeyDown = (event: KeyboardEvent) => {
  if (event.key === "Enter") {
    event.preventDefault();
    const activeHint = hints.value[activeHintIndex.value];
    /* v8 ignore if -- @preserve */
    if (!activeHint) return;
    confirmStep(activeHint);
  }
  if (event.key === "Escape") {
    event.preventDefault();
    resetInput();
  }
  if (event.key === "Tab") {
    onFocus(false);
  }
  if (event.key === "ArrowDown") {
    event.preventDefault();
    activeHintIndex.value = (activeHintIndex.value + 1) % hints.value.length;
  }
  if (event.key === "ArrowUp") {
    event.preventDefault();
    activeHintIndex.value =
      (activeHintIndex.value - 1 + hints.value.length) % hints.value.length;
  }
};

const followers = computed<QueryType[]>(() => {
  return queryApiMock[props.prevStep?.type ?? ""].followers.filter(
    (f): f is QueryType => Object.keys(queryApiMock).includes(f),
  ) as QueryType[];
});

const hints = computed<QueryType[]>(() => {
  return followers.value.filter((f) =>
    content.value.length === 0
      ? true
      : f.toLowerCase().startsWith(content.value.toLowerCase()),
  );
});

const emit = defineEmits<{
  (e: "confirm-step", stepType: QueryType): void;
}>();

const confirmStep = (stepType: QueryType) => {
  emit("confirm-step", stepType);
  resetInput();
  contentInput.value?.focus();
};

const activeHintIndex = ref(0);

const resetInput = () => {
  content.value = "";
  activeHintIndex.value = 0;
  /* v8 ignore else -- @preserve */
  if (contentInput.value) {
    contentInput.value.innerText = "";
  }
};

const hasFocus = ref(false);
const displayHints = computed(() => {
  return hints.value.length > 0 && hasFocus.value;
});

const onFocus = (focus: boolean) => {
  hasFocus.value = focus;
};
</script>

<template>
  <div
    v-if="followers.length > 0"
    v-on-click-outside="() => onFocus(false)"
    class="query-step-input"
    @focusin="onFocus(true)"
  >
    <div
      ref="contentInput"
      class="step-input"
      contenteditable="true"
      role="textbox"
      aria-label="Query input"
      aria-multiline="false"
      spellcheck="false"
      aria-autocomplete="list"
      aria-haspopup="listbox"
      :aria-expanded="displayHints"
      @input.stop="updateContent"
      @keydown.stop="onKeyDown"
    ></div>
    <FadeTransition>
      <QueryHinter
        v-if="displayHints"
        :hints="hints"
        :active-index="activeHintIndex"
        @select-hint="confirmStep"
      />
    </FadeTransition>
  </div>
</template>

<style lang="less" scoped>
.query-step-input {
  position: relative;
  display: inline-block;
  flex: 1;
  min-width: 10rem;
}
</style>
