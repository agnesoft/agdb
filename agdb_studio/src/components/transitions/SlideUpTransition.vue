<template>
    <transition name="slide-up">
        <slot></slot>
    </transition>
</template>

<style lang="less" scoped>
.slide-up-enter-active,
.slide-up-leave-active {
    transition:
        max-height 0.25s ease,
        opacity 0.25s ease;
    max-height: 500px;
    overflow: hidden;
}

.slide-up-enter-from,
.slide-up-leave-to {
    opacity: 0;
    max-height: 0;
}
</style>
