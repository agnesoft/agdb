type StyleObject = {
    [key: string]: string;
};

type ActionFn<T> = ((params: T) => void) | (() => void);

type Action = {
    key: string;
    label: string;
    action?: ActionFn;
    actions?: Action[];
};
