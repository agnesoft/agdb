<script lang="ts" setup>
import useModal from "../../composables/modal/modal";
import { ClCloseMd } from "@kalimahapps/vue-icons";
import AgdbContent from "../content/AgdbContent.vue";
import { KEY_MODAL } from "../../composables/modal/constants";
import { nextTick, ref, watch } from "vue";

const { modal, buttons, closeModal, modalIsVisible } = useModal();

const autofocusElement = ref();

watch(modalIsVisible, async () => {
  if (
    !modalIsVisible.value ||
    modal.content.some((part) => part.input?.autofocus)
  ) {
    return;
  }
  await nextTick();
  autofocusElement.value?.focus();
});
</script>

<template>
  <div v-if="modalIsVisible" class="modal-background">
    <section class="modal">
      <header class="modal-header">
        <h3>{{ modal.header }}</h3>
        <button
          class="button button-transparent"
          data-testid="close-modal"
          @click="closeModal"
        >
          <ClCloseMd />
        </button>
      </header>
      <form id="modal-form">
        <AgdbContent
          :content="modal.content"
          :content-key="KEY_MODAL"
          class="modal-body"
        />
      </form>
      <footer class="modal-footer">
        <button
          v-for="button in buttons"
          :key="button.text"
          :ref="
            (el) => {
              if (button.type === 'submit') autofocusElement = el;
            }
          "
          :class="button.className"
          :type="button.type ?? 'button'"
          :form="button.type === 'submit' ? 'modal-form' : undefined"
          @click.prevent="button.action"
        >
          {{ button.text }}
        </button>
      </footer>
    </section>
  </div>
</template>

<style lang="less" scoped>
.modal-background {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: var(--z-index-modal);
}
.modal {
  background-color: var(--color-background-soft);
  border: 1px solid var(--color-border);
  border-radius: 5px;
  box-shadow: 0 0 10px rgba(0, 0, 0, 0.5);
  max-width: 90%;
  width: 30rem;
}
.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.6rem 1rem;
  border-bottom: 1px solid var(--color-border);
}
.modal-body {
  padding: 1rem;
  p {
    margin-bottom: 1rem;
  }
}
.modal-footer {
  display: flex;
  justify-content: flex-end;
  padding: 1rem;
  border-top: 1px solid var(--color-border);
  gap: 1rem;
}
</style>
