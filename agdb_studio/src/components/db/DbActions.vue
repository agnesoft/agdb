<script lang="ts" setup>
import type { Action, TRow } from "@/composables/table/types";
import AgdbCellMenu from "../base/table/AgdbCellMenu.vue";

const actions: Action<TRow>[] = [
    { label: "Edit", action: () => console.log("Edit") },
    { label: "Delete", action: () => console.log("Delete") },
];
</script>

<template>
    <AgdbCellMenu :actions="actions" />
</template>

<style lang="less" scoped></style>
