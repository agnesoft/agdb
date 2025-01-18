<script lang="ts" setup>
import { vOnClickOutside } from "@vueuse/components";
import { ref } from "vue";
import DropdownContent from "./DropdownContent.vue";
import SlideUpTransition from "@/components/transitions/SlideUpTransition.vue";

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
        <button type="button" class="trigger" @click="toggle" ref="buttonRef">
            <slot name="trigger"></slot>
        </button>
        <Teleport to="body">
            <SlideUpTransition>
                <DropdownContent
                    :button-ref="buttonRef"
                    :opened="opened"
                    v-on-click-outside="close"
                    @close="close"
                >
                    <slot name="content"></slot>
                </DropdownContent>
            </SlideUpTransition>
        </Teleport>
    </div>
</template>

<style lang="less" scoped>
.trigger {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
}
</style>
