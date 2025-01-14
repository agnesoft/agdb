<script lang="ts" setup>
import type { TRow } from "@/composables/table/types";
import AgdbDropdown from "../dropdown/AgdbDropdown.vue";
import { MdRoundMenu } from "@kalimahapps/vue-icons";
import { computed, inject, type PropType, type Ref } from "vue";
import { INJECT_KEY_ROW } from "@/composables/table/constants";
import AgdbMenu from "../menu/AgdbMenu.vue";
import { useDbStore } from "@/composables/db/dbStore";
import useModal from "@/composables/modal/modal";

const props = defineProps({
    actions: { type: Array as PropType<Action[]>, required: true },
});

const row = inject<Ref<TRow>>(INJECT_KEY_ROW);
const { fetchDatabases } = useDbStore();
const { openModal } = useModal();

const mapActions = (actions: Action[]): Action[] => {
    return actions.map((action) => {
        const runAction = action.action
            ? ({ event }: ActionProps<undefined>) => {
                  action.action?.({ event, params: row?.value });
                  fetchDatabases();
              }
            : undefined;
        return {
            ...action,
            action: !runAction
                ? ({ event }: ActionProps<undefined>) => {
                      event.preventDefault();
                      event.stopPropagation();
                  }
                : action.confirmation
                  ? ({ event }: ActionProps<undefined>) =>
                        openModal({
                            header: action.confirmationHeader
                                ? typeof action.confirmationHeader ===
                                  "function"
                                    ? action.confirmationHeader({
                                          params: row?.value,
                                      })
                                    : action.confirmationHeader
                                : "Confirm action",
                            content: action.confirmation,
                            onConfirm: () =>
                                runAction({ event, params: undefined }),
                        })
                  : runAction,
            actions: action.actions ? mapActions(action.actions) : undefined,
        };
    });
};
const actions = computed(() => mapActions(props.actions));
</script>

<template>
    <AgdbDropdown>
        <template #trigger>
            <div class="button"><MdRoundMenu /></div>
        </template>
        <template #content>
            <AgdbMenu :actions="actions" />
        </template>
    </AgdbDropdown>
</template>

<style lang="less" scoped></style>
