<script lang="ts" setup>
import FadeTransition from "@agdb-studio/design/src/components/transitions/FadeTransition.vue";
import { PhDuotoneSpinnerBall } from "@kalimahapps/vue-icons";
</script>

<template>
  <FadeTransition>
    <span class="spinner-icon">
      <PhDuotoneSpinnerBall />
    </span>
  </FadeTransition>
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
.spinner-icon {
  display: inline-flex;
  justify-content: center;
  align-items: center;
}
.spinner-icon > svg {
  animation: spin 1.2s linear infinite;
}
</style>
