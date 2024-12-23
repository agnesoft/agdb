<script lang="ts" setup>
import { vOnClickOutside } from "@vueuse/components";
import { ref } from "vue";

const opened = ref(false);
const close = () => {
    if (!opened.value) return;
    opened.value = false;
};
const toggle = () => {
    opened.value = !opened.value;
};
</script>

<template>
    <div class="agdb-dropdown" v-on-click-outside="close">
        <button type="button" class="trigger button" @click="toggle">
            <slot name="trigger"></slot>
        </button>
        <div v-if="opened" class="content">
            <slot name="content"></slot>
        </div>
    </div>
</template>

<style lang="less" scoped>
.content {
    position: absolute;
    background-color: var(--color-background-mute);
    min-width: 160px;
    box-shadow: 0 8px 16px 0 rgba(0, 0, 0, 0.2);
    z-index: 1;
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    overflow: auto;
}
</style>
