import { computed, reactive, ref } from "vue";
import type { Button, Modal } from "./types";
import { useContentInputs } from "../content/inputs";
import { KEY_MODAL } from "./constants";

const { addInput, clearInputs } = useContentInputs();
const modal = reactive<Modal>({
    header: "",
    content: [],
});

const modalIsVisible = ref(false);

const onConfirm = ref<() => void>();

const hideModal = (): void => {
    modal.header = "";
    modal.content = [];
    modalIsVisible.value = false;
    clearInputs(KEY_MODAL);
};

const customButtons = ref<Button[]>([]);

const buttons = computed<Button[]>(() => {
    const defaultButtons: Button[] = [
        {
            className: "button",
            text: "Close",
            action: hideModal,
        },
    ];
    if (onConfirm.value) {
        defaultButtons.push({
            className: "button button-success",
            text: "Confirm",
            action: () => {
                onConfirm.value?.();
                hideModal();
            },
            type: "submit",
        });
    }
    return [...customButtons.value, ...defaultButtons];
});

type ShowModalProps = {
    header?: string;
    content?: Content[];
    onConfirm?: () => void;
    buttons?: Button[];
};

const showModal = ({
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
            addInput(KEY_MODAL, c.input.key, ref());
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
        hideModal,
        showModal,
        onConfirm,
    };
}
