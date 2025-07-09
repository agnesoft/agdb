<script lang="ts" setup>
import type { TRow } from "../../composables/table/types";
import AgdbDropdown from "../dropdown/AgdbDropdown.vue";
import { MdRoundMenu } from "@kalimahapps/vue-icons";
import { computed, inject, type PropType, type Ref } from "vue";
import {
  INJECT_KEY_ROW,
  INJECT_KEY_TABLE_NAME,
} from "../../composables/table/constants";
import AgdbMenu from "../menu/AgdbMenu.vue";
import useModal from "../../composables/modal/modal";
import { fetchData } from "../../composables/table/tableConfig";
import type {
  Action,
  ActionFn,
  ActionProps,
  ActionReturn,
} from "../../composables/content/types";

const props = defineProps({
  actions: { type: Array as PropType<Action<TRow>[]>, required: true },
});

const row = inject<Ref<TRow>>(INJECT_KEY_ROW);
const tableName = inject<Ref<string | symbol>>(INJECT_KEY_TABLE_NAME);

const { openModal } = useModal();

const mapActions = (actions: Action<TRow>[]): Action<undefined>[] => {
  if (!row) return [];
  return actions.map((action) => {
    const runAction: ActionFn<undefined, ActionReturn> | undefined =
      action.action
        ? async ({
            event,
          }: ActionProps<undefined>): Promise<boolean | void> => {
            /* v8 ignore next */
            if (!action.action) return false;
            const result = await action.action({
              event,
              params: row?.value,
            });
            await fetchData(tableName?.value);
            return result;
          }
        : undefined;
    return {
      key: action.key,
      label: action.label,
      action: !runAction
        ? ({ event }: ActionProps<undefined>) => {
            event.preventDefault();
            event.stopPropagation();
            return false;
          }
        : action.confirmation
          ? ({ event }: ActionProps<undefined>) => {
              openModal({
                header: action.confirmationHeader
                  ? typeof action.confirmationHeader === "function"
                    ? action.confirmationHeader({
                        event,
                        params: row.value,
                      })
                    : action.confirmationHeader
                  : "Confirm action",
                content:
                  typeof action.confirmation === "function"
                    ? action.confirmation({
                        event,
                        params: row.value,
                      })
                    : action.confirmation,
                onConfirm: () =>
                  runAction({
                    event,
                    params: undefined,
                  }),
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
