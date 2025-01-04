<script lang="ts" setup>
import { ref, type PropType } from "vue";
import { AkChevronRightSmall } from "@kalimahapps/vue-icons";
import FadeTrasition from "@/components/transitions/FadeTrasition.vue";

const props = defineProps({
    actions: { type: Array as PropType<Action[]>, required: true },
});

const openedSubmenu = ref<string>();
const openSubmenu = (key: string) => {
    openedSubmenu.value = key;
};
</script>

<template>
    <ul class="agdb-menu" @mouseleave="openedSubmenu = undefined">
        <li
            v-for="action in props.actions"
            :key="action.key"
            @click.prevent="
                (event: MouseEvent) => {
                    if (action.actions) {
                        openSubmenu(action.key);
                    }
                    action.action({ event });
                }
            "
            class="menu-item"
            @mouseover="openSubmenu(action.key)"
            :data-key="action.key"
        >
            <a
                href="#"
                :class="{
                    active: openedSubmenu === action.key && action.actions,
                }"
            >
                {{ action.label }}
                <span v-if="action.actions" class="menu-item-button">
                    <AkChevronRightSmall />
                </span>
            </a>
            <FadeTrasition>
                <AgdbMenu
                    class="sub-menu"
                    v-if="openedSubmenu === action.key && action.actions"
                    :actions="action.actions"
                />
            </FadeTrasition>
        </li>
    </ul>
</template>

<style lang="less" scoped>
.menu-item {
    cursor: pointer;
    transition:
        background-color 0.2s,
        color 0.2s;
    position: relative;
    &:hover {
        background-color: var(--color-background-active);
        color: var(--black);
    }
    &:first-child {
        border-top-left-radius: 0.5rem;
        border-top-right-radius: 0.5rem;
    }
    &:last-child {
        border-bottom-left-radius: 0.5rem;
        border-bottom-right-radius: 0.5rem;
    }

    a {
        padding: 0.5rem;
        display: block;
        color: var(--color-text);
        text-decoration: none;
        opacity: none;
        transition: color 0.2s;
        width: 100%;
        height: 100%;
        &:hover,
        &.active {
            color: var(--black);
        }
    }
}
.menu-item-button {
    float: right;
}
.agdb-menu,
::v-deep(.agdb-menu) {
    color: var(--color-text);
    background-color: var(--color-background-mute);
    min-width: 10rem;
    box-shadow: 0 8px 16px 0 rgba(0, 0, 0, 0.2);
    z-index: 1;
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
}
.sub-menu {
    position: absolute;
    left: calc(100% - 2rem);
    top: 0.5rem;
}
</style>
