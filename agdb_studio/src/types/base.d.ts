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
type Content = {
    paragraph?: Paragraph[];
    component?: string;
};

type Action = {
    key: string;
    label: string;
    action?: ActionFn;
    actions?: Action[];
    confirmation?: Content[];
};
