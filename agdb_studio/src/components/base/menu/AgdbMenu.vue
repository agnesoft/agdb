<script lang="ts" setup>
import { ref, type PropType } from "vue";
import { AkChevronRightSmall } from "@kalimahapps/vue-icons";

const props = defineProps({
    actions: { type: Array as PropType<Action[]>, required: true },
});

const openedSubmenu = ref<string>();
const openSubmenu = (label: string) => {
    openedSubmenu.value = label;
};
</script>

<template>
    <div class="agdb-menu">
        <div
            v-for="action in props.actions"
            :key="action.label"
            @click="action.action"
            class="menu-item"
            @hover="openSubmenu(action.label)"
        >
            {{ action.label }}
            <button v-if="action.actions" class="menu-item-button">
                <AkChevronRightSmall />
            </button>
            <AgdbMenu
                v-if="openedSubmenu === action.label && action.actions"
                :actions="action.actions"
            />
            <!-- <div v-if="openedSubmenu === action.label" class="submenu">
                <div
                    v-for="submenuAction in action.actions"
                    :key="submenuAction.label"
                    @click="submenuAction.action"
                    class="submenu-item"
                >
                    {{ submenuAction.label }}
                </div>
            </div> -->
        </div>
    </div>
</template>

<style lang="less" scoped></style>
