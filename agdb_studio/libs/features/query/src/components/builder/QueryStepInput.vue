<script lang="ts" setup>
import { computed, ref } from "vue";
import { queryApiMock } from "../../mock/queryApiMock";
import type { QueryStep, QueryType } from "../../composables/types";
import QueryHinter from "./QueryHinter.vue";

const props = defineProps<{
  prevStep?: QueryStep;
}>();

const content = ref("");
const contentInput = ref<HTMLElement | null>(null);

const updateContent = () => {
  content.value =
    contentInput.value?.innerText.trim().replace(/[^\w]/g, "") ?? "";
};

const onKeyDown = (event: KeyboardEvent) => {
  if (["Enter", "Tab"].includes(event.key)) {
    event.preventDefault();
    confirmStep();
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

const hints = computed<QueryType[]>(() => {
  const followers: QueryType[] = queryApiMock[
    props.prevStep?.type.length ? props.prevStep.type : ""
  ].followers.filter((f): f is QueryType =>
    Object.keys(queryApiMock).includes(f),
  ) as QueryType[];

  return followers.filter((f) =>
    content.value.length === 0
      ? true
      : f.toLowerCase().startsWith(content.value.toLowerCase()),
  );
});

const emit = defineEmits<{
  (e: "confirm-step", stepType: QueryType): void;
}>();

const confirmStep = () => {
  const stepType = hints.value[activeHintIndex.value];
  if (stepType) {
    emit("confirm-step", stepType);
    content.value = "";
    activeHintIndex.value = 0;
    if (contentInput.value) {
      contentInput.value.innerText = "";
    }
  }
};

const activeHintIndex = ref(0);
</script>

<template>
  <div class="query-step-input">
    <div
      ref="contentInput"
      class="step-input"
      contenteditable="true"
      role="textbox"
      aria-label="Query input"
      aria-multiline="false"
      spellcheck="false"
      @input="updateContent"
      @keydown="onKeyDown"
    ></div>
    <QueryHinter
      :hints="hints"
      :activeIndex="activeHintIndex"
      @selectHint="confirmStep"
    />
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
