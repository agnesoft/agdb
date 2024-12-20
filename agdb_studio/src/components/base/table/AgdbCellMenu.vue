<script lang="ts" setup>
import AgdbDropdown from "../dropdown/AgdbDropdown.vue";
import { MdRoundMenu } from "@kalimahapps/vue-icons";
// import type { TRow } from "@/composables/table/types";
// import { inject } from "vue";
</script>

<template>
    <AgdbDropdown>
        <template #trigger>
            <button class="button"><MdRoundMenu /></button>
        </template>
        <template #content>
            <div class="agdb-cell-actions-dropdown">
                <slot></slot>
            </div>
        </template>
    </AgdbDropdown>
</template>

<style lang="less" scoped></style>
