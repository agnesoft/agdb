<template>
    <transition name="fade">
        <slot></slot>
    </transition>
</template>

<style lang="less" scoped>
.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.25s ease;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}
</style>
