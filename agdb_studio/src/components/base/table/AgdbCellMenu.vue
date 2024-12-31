<script lang="ts" setup>
import type { TRow } from "@/composables/table/types";
import AgdbDropdown from "../dropdown/AgdbDropdown.vue";
import { MdRoundMenu } from "@kalimahapps/vue-icons";
import { computed, inject, type PropType, type Ref } from "vue";
import { INJECT_KEY_ROW } from "@/composables/table/constants";
import AgdbMenu from "../menu/AgdbMenu.vue";
import { useDbStore } from "@/composables/db/dbStore";

const props = defineProps({
    actions: { type: Array as PropType<Action[]>, required: true },
});

const row = inject<Ref<TRow>>(INJECT_KEY_ROW);
const { fetchDatabases } = useDbStore();

const mapActions = (actions: Action[]): Action[] => {
    return actions.map((action) => {
        return {
            ...action,
            action: action.action
                ? ({ event }: ActionProps<undefined>) => {
                      action.action?.({ event, params: row?.value });
                      fetchDatabases();
                  }
                : ({ event }: ActionProps<undefined>) => {
                      event.preventDefault();
                      event.stopPropagation();
                  },
            actions: action.actions ? mapActions(action.actions) : undefined,
        };
    });
};
const actions = computed(() => mapActions(props.actions));
</script>

<template>
    <AgdbDropdown>
        <template #trigger>
            <MdRoundMenu />
        </template>
        <template #content>
            <AgdbMenu :actions="actions" />
        </template>
    </AgdbDropdown>
</template>

<style lang="less" scoped></style>
