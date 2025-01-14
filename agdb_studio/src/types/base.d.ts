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
type InputType =
    | "text"
    | "number"
    | "password"
    | "email"
    | "checkbox"
    | "select";
type OptionType = {
    value: string;
    label: string;
};
type Input = {
    key: string;
    label: string;
    type: InputType;
    style?: StyleObject;
    className?: string;
    autofocus?: boolean;
    options?: OptionType[];
    defaultValue?: string | number | boolean;
    required?: boolean;
};

type Content = {
    paragraph?: Paragraph[];
    component?: AsyncComponent;
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
