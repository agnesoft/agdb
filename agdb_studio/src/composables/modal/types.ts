export type Modal = {
    header: string;
    content: Content[];
};

export type Button = {
    className: string;
    text: string;
    action: () => void | Promise<void>;
    type?: "button" | "submit" | "reset";
};
