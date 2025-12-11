<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { codeToHtml } from "shiki";
import { useColorMode } from "#imports";
import openapi from "../../../../agdb_server/openapi.json";

const code = ref<string>();
const isExpanded = ref(false);
const highlightedHtml = ref<string>();
const colorMode = useColorMode();

const highlightCode = async () => {
  /* v8 ignore if -- @preserve */
  if (!code.value) return;

  const theme = colorMode.value === "dark" ? "github-dark" : "github-light";

  highlightedHtml.value = await codeToHtml(code.value, {
    lang: "json",
    theme,
  });
};

onMounted(async () => {
  try {
    code.value = JSON.stringify(openapi, null, 2);
    await highlightCode();
  } catch (error) {
    console.error("Failed to load openapi.json:", error);
  }
});

watch(() => colorMode.value, highlightCode);

const copyCode = () => {
  /* v8 ignore else -- @preserve */
  if (code.value) {
    navigator.clipboard.writeText(code.value);
  }
};
</script>

<template>
  <div v-if="code" class="openapi-wrapper">
    <UButton v-if="!isExpanded" size="xl" @click="isExpanded = true">
      Show openapi.json
    </UButton>
    <div v-else class="code-block">
      <div class="code-header">
        <span>openapi.json</span>
        <div class="actions">
          <UButton
            size="xs"
            variant="ghost"
            icon="i-lucide-copy"
            aria-label="Copy code"
            @click="copyCode"
          />
          <UButton
            data-testid="hide-button"
            size="sm"
            variant="outline"
            @click="isExpanded = false"
          >
            Hide
          </UButton>
        </div>
      </div>
      <!-- eslint-disable-next-line vue/no-v-html -->
      <div class="code-content" v-html="highlightedHtml"></div>
    </div>
  </div>
</template>

<style lang="less">
.openapi-wrapper {
  margin: 2rem 0;
}

.code-block {
  border: 1px solid var(--ui-border);
  border-radius: var(--ui-radius);
  overflow: hidden;

  .code-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--ui-border);
    background: var(--ui-bg-elevated);
    font-family: var(--font-mono);
    font-size: 0.875rem;

    .actions {
      display: flex;
      gap: 0.5rem;
      align-items: center;
    }
  }

  pre {
    margin: 0;
    padding: 1rem;
    overflow-x: auto;
    max-height: 80vh;
    overflow-y: auto;
    background: var(--ui-bg);

    code {
      font-family: var(--font-mono);
      font-size: 0.875rem;
      line-height: 1.7;
      color: var(--ui-text);
    }
  }
}
</style>
