<script lang="ts" setup>
import { vOnClickOutside } from "@vueuse/components";
import { ref } from "vue";
import DropdownContent from "./DropdownContent.vue";

const opened = ref(false);
const close = () => {
    opened.value = false;
};
const toggle = () => {
    opened.value = !opened.value;
};
const buttonRef = ref<HTMLElement>();
</script>

<template>
    <div class="agdb-dropdown">
        <button
            type="button"
            class="trigger button"
            @click="toggle"
            ref="buttonRef"
        >
            <slot name="trigger"></slot>
        </button>
        <Teleport to="body">
            <DropdownContent
                :button-ref="buttonRef"
                :opened="opened"
                v-on-click-outside="close"
                @close="close"
            >
                <slot name="content"></slot>
            </DropdownContent>
        </Teleport>
    </div>
</template>

<style lang="less" scoped></style>
