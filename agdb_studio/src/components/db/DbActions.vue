<script lang="ts" setup>
import { inject } from "vue";
import AgdbCellMenu from "../base/table/AgdbCellMenu.vue";

const row = inject("row");
</script>

<template>
    <AgdbCellMenu></AgdbCellMenu>
</template>

<style lang="less" scoped></style>
