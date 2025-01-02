import { computed, reactive, ref } from "vue";
import type { Button, Modal } from "./types";

const modal = reactive<Modal>({
    header: "",
    content: [],
});

const modalIsVisible = ref(false);

const onConfirm = ref<() => void>();

const hideModal = () => {
    modal.header = "";
    modal.content = [];
    modalIsVisible.value = false;
};

const customButtons = ref<Button[]>([]);

const buttons = computed<Button[]>(() => {
    const defaultButtons = [
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
}: ShowModalProps) => {
    modal.header = header ?? "";
    modal.content = content ?? [];
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
