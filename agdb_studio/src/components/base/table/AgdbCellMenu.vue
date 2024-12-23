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
            <MdRoundMenu />
        </template>
        <template #content>
            <div>
                <div
                    v-for="action in props.actions"
                    :key="action.label"
                    @click="() => action.action(row)"
                    class="dropdown-item"
                >
                    {{ action.label }}
                </div>
            </div>
        </template>
    </AgdbDropdown>
</template>

<style lang="less" scoped>
.dropdown-item {
    padding: 0.5rem;
    cursor: pointer;
    transition:
        background-color 0.2s,
        color 0.2s;
    &:hover {
        background-color: var(--color-background-active);
        color: var(--black);
    }
}
</style>
