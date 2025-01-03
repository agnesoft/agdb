type StyleObject = {
    [key: string]: string;
};

type ActionProps<T> = {
    event: Event;
    params: T;
};
type ActionFn<T> = ({ event, params }: ActionProps<T>) => void;

type Paragraph = {
    text: string;
    style?: StyleObject;
    className?: string;
};
type InputType = "text" | "number" | "password" | "email" | "checkbox";
type Input = {
    key: string;
    label: string;
    type: InputType;
    style?: StyleObject;
    className?: string;
    autofocus?: boolean;
};
type Content = {
    paragraph?: Paragraph[];
    component?: string;
    input?: Input;
};

type Action = {
    key: string;
    label: string;
    action?: ActionFn;
    actions?: Action[];
    confirmation?: Content[];
    confirmationHeader?: string | ActionFn;
};
