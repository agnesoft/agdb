<script lang="ts" setup>
import { vOnClickOutside } from "@vueuse/components";
import { ref } from "vue";

const opened = ref(false);
const close = () => {
    opened.value = false;
};
const toggle = () => {
    opened.value = !opened.value;
};
</script>

<template>
    <div class="agdb-dropdown">
        <button type="button" class="trigger" @click="toggle">
            <slot name="trigger"></slot>
        </button>
        <div v-if="opened" class="content" v-on-click-outside="close">
            <slot name="content"></slot>
        </div>
    </div>
</template>

<style lang="less" scoped></style>
