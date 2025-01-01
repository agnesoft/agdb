import { computed, reactive, ref } from "vue";

type Modal = {
    header: string;
    body: string[];
};
const content = reactive<Modal>({
    header: "",
    body: [],
});

const modalIsVisible = ref(false);

const onConfirm = ref<() => void>();

const hideModal = () => {
    content.header = "";
    content.body = [];
    modalIsVisible.value = false;
};

type Button = {
    className: string;
    text: string;
    action: () => void;
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
    body?: string[];
    footer?: string;
    onConfirm?: () => void;
    buttons?: Button[];
};

const showModal = ({
    header,
    body,
    onConfirm: onConfirmFn,
    buttons: extraButtons,
}: ShowModalProps) => {
    content.header = header ?? "";
    content.body = body ?? [];
    onConfirm.value = onConfirmFn;
    modalIsVisible.value = true;
    customButtons.value = extraButtons || [];
};

export default function useModal() {
    return {
        content,
        buttons,
        modalIsVisible,
        hideModal,
        showModal,
        onConfirm,
    };
}
