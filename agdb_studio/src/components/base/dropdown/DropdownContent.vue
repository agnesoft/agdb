<script lang="ts" setup>
import { computed, ref } from "vue";

const props = defineProps({
    opened: { type: Boolean, required: true },
    buttonRef: { type: HTMLElement, required: false },
});

const contentRef = ref<HTMLElement>();

const contentStyle = computed(() => {
    if (!props.opened || !props.buttonRef || !contentRef.value) return {};
    const rect = props.buttonRef.getBoundingClientRect();
    let left = rect.left;
    let top = props.buttonRef.offsetTop + props.buttonRef.offsetHeight;
    const contentRect = contentRef.value.getBoundingClientRect();
    if (left + contentRect.width > window.innerWidth) {
        left = window.innerWidth - contentRect.width;
    }
    if (
        top + contentRect.height > window.innerHeight &&
        props.buttonRef.offsetTop - contentRect.height > 0
    ) {
        top = props.buttonRef.offsetTop - contentRect.height;
    }

    return {
        left: `${left}px`,
        top: `${top}px`,
    };
});
</script>

<template>
    <div
        class="content"
        @click="$emit('close')"
        v-if="opened"
        :style="contentStyle"
        ref="contentRef"
    >
        <slot></slot>
    </div>
</template>
<style lang="less" scoped>
.content {
    position: absolute;
    z-index: 100;
}
</style>
