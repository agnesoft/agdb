export type Modal = {
    header: string;
    content: Content[];
};

export type Button = {
    className: string;
    text: string;
    action: () => void;
    type?: "button" | "submit" | "reset";
};
