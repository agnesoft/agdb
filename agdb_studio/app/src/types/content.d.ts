type ActionProps<T> = {
  event: Event;
  params: T;
};
type ActionFn<P, R> = ({ event, params }: ActionProps<P>) => R;

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
  required?: boolean;
  value?: string | number | boolean;
  error?: string;
  rules?: ((value: string) => string | undefined)[];
};

type Content = {
  paragraph?: Paragraph[];
  component?: AsyncComponent;
  input?: Input;
};

type ActionReturn = Promise<boolean | void> | boolean;

type Action<P> = {
  key: string;
  label: string;
  action?: ActionFn<P, ActionReturn>;
  actions?: Action[];
  confirmation?: Content[] | ActionFn<P, Content[]>;
  confirmationHeader?: string | ActionFn<P, string>;
};
