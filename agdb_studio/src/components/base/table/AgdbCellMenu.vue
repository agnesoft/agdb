<script lang="ts" setup>
import type { Action, TRow } from "@/composables/table/types";
import AgdbDropdown from "../dropdown/AgdbDropdown.vue";
import { MdRoundMenu } from "@kalimahapps/vue-icons";
import { inject, type PropType } from "vue";
import { INJECT_KEY_ROW } from "@/composables/table/constants";

const props = defineProps({
    actions: { type: Array as PropType<Action<TRow>[]>, required: true },
});

const row = inject<TRow>(INJECT_KEY_ROW)!;
</script>

<template>
    <AgdbDropdown>
        <template #trigger>
            <button class="button"><MdRoundMenu /></button>
        </template>
        <template #content>
            <div class="agdb-cell-actions-dropdown">
                <div
                    v-for="action in props.actions"
                    :key="action.label"
                    @click="() => action.action(row)"
                    class="button"
                >
                    {{ action.label }}
                </div>
            </div>
        </template>
    </AgdbDropdown>
</template>

<style lang="less" scoped></style>
