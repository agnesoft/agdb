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
    actions: { type: Array as PropType<Action<any>[]>, required: true },
});

const row = inject<Ref<TRow>>(INJECT_KEY_ROW);
const { fetchDatabases } = useDbStore();
const { openModal } = useModal();

const mapActions = (actions: Action<TRow>[]): Action<TRow>[] => {
    return actions.map((action) => {
        const runAction: ActionFn<TRow, ActionReturn> | undefined =
            action.action
                ? ({ event }: ActionProps<TRow>): ActionReturn => {
                      if (!row || !action.action) return false;
                      const result = action.action({
                          event,
                          params: row?.value,
                      });
                      fetchDatabases();
                      return result;
                  }
                : undefined;
        return {
            ...action,
            action: !runAction
                ? ({ event }: ActionProps<TRow>) => {
                      event.preventDefault();
                      event.stopPropagation();
                      return false;
                  }
                : action.confirmation
                  ? ({ event }: ActionProps<TRow>) => {
                        openModal({
                            header:
                                action.confirmationHeader && row !== undefined
                                    ? typeof action.confirmationHeader ===
                                      "function"
                                        ? action.confirmationHeader({
                                              event,
                                              params: row.value,
                                          })
                                        : action.confirmationHeader
                                    : "Confirm action",
                            content:
                                action.confirmation && row !== undefined
                                    ? typeof action.confirmation === "function"
                                        ? action.confirmation({
                                              event,
                                              params: row.value,
                                          })
                                        : action.confirmation
                                    : undefined,
                            onConfirm: () => {
                                if (row !== undefined)
                                    runAction({
                                        event,
                                        params: row.value,
                                    });
                                return true;
                            },
                        });
                        return false;
                    }
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
