<script lang="ts" setup>
import { vOnClickOutside } from "@vueuse/components";
import { ref } from "vue";
import DropdownContent from "./DropdownContent.vue";
import SlideUpTransition from "@agdb-studio/design/src/components/transitions/SlideUpTransition.vue";

const opened = ref(false);
const close = () => {
  opened.value = false;
};
const toggle = (event: MouseEvent) => {
  event.stopPropagation();
  opened.value = !opened.value;
};
const buttonRef = ref<HTMLElement>();
</script>

<template>
  <div class="agdb-dropdown">
    <button
      ref="buttonRef"
      type="button"
      class="trigger"
      @click="toggle"
      title="Display menu"
      aria-haspopup="true"
      :aria-expanded="opened"
      aria-controls="dropdown-content"
    >
      <slot name="trigger"></slot>
    </button>
    <Teleport to="#app">
      <SlideUpTransition>
        <DropdownContent
          v-on-click-outside="close"
          :button-ref="buttonRef"
          :opened="opened"
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
