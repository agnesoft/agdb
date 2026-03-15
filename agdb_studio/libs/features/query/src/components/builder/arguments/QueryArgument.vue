<script lang="ts" setup>
import type { QueryArguments } from "@/mock/queryApiMock";

const props = defineProps<{
  arguments: QueryArguments;
}>();
</script>

<template>
  <div class="query-argument"></div>
</template>

<style lang="less" scoped></style>
