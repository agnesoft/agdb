<script lang="ts" setup>
import { PhDuotoneSpinnerBall } from "@kalimahapps/vue-icons";
</script>

<template>
    <span class="spinner-icon">
        <PhDuotoneSpinnerBall />
    </span>
</template>

<style lang="less" scoped>
@keyframes spin {
    0% {
        transform: rotate(0deg);
    }
    100% {
        transform: rotate(360deg);
    }
}
.spinner-icon > svg {
    animation: spin 1.2s linear infinite;
}
</style>
