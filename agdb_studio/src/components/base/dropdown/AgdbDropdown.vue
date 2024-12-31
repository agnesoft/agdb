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
        <div v-if="opened" class="content" @click="close">
            <slot name="content"></slot>
        </div>
    </div>
</template>

<style lang="less" scoped>
.content {
    position: absolute;
}
</style>
