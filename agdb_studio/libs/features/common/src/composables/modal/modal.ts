import { computed, reactive, ref } from "vue";
import type { Button, Modal } from "./types";
import { useContentInputs } from "../content/inputs";
import { KEY_MODAL } from "./constants";

const { addInput, clearInputs, checkInputsRules } = useContentInputs();
const modal = reactive<Modal>({
  header: "",
  content: [],
});

const modalIsVisible = ref(false);

const onConfirm = ref<() => Promise<void | boolean> | boolean>();

const closeModal = (): void => {
  modal.header = "";
  modal.content = [];
  modalIsVisible.value = false;
  clearInputs(KEY_MODAL);
};

const customButtons = ref<Button[]>([]);

const confirmLoading = ref(false);

const handleConfirm = async (): Promise<void> => {
  if (!checkInputsRules(KEY_MODAL) || !onConfirm.value) {
    return;
  }
  confirmLoading.value = true;
  const result = onConfirm.value();
  if (result instanceof Promise) {
    result.then(
      (res: void | boolean) => {
        confirmLoading.value = false;
        if (res !== false) closeModal();
      },
      () => {
        confirmLoading.value = false;
      },
    );
    return;
  } else if (result) {
    confirmLoading.value = false;
    closeModal();
  }
};

const buttons = computed<Button[]>(() => {
  const defaultButtons: Button[] = [
    {
      className: "button",
      text: "Close",
      action: closeModal,
    },
  ];
  if (onConfirm.value) {
    defaultButtons.push({
      className: "button button-success",
      text: "Confirm",
      action: handleConfirm,
      type: "submit",
    });
  }
  return [...customButtons.value, ...defaultButtons];
});

type ShowModalProps = {
  header?: string;
  content?: Content[];
  onConfirm?: () => Promise<void | boolean> | boolean;
  buttons?: Button[];
};

const openModal = ({
  header,
  content,
  onConfirm: onConfirmFn,
  buttons: extraButtons,
}: ShowModalProps): void => {
  modal.header = header ?? "";
  modal.content = content ?? [];
  clearInputs(KEY_MODAL);
  content?.forEach((c) => {
    if (c.input) {
      addInput(KEY_MODAL, c.input);
    }
  });

  onConfirm.value = onConfirmFn;
  modalIsVisible.value = true;
  customButtons.value = extraButtons || [];
};

export default function useModal() {
  return {
    modal,
    buttons,
    modalIsVisible,
    closeModal,
    openModal,
    onConfirm,
    handleConfirm,
  };
}
