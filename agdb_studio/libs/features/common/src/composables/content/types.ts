import type { StyleObject } from "@agdb-studio/design/src/types/base";
import type { AsyncComponent } from "../../types/asyncComponents";

export type ActionProps<T> = {
  event: Event;
  params: T;
};
export type ActionFn<P, R> = ({ event, params }: ActionProps<P>) => R;

export type Paragraph = {
  text: string;
  style?: StyleObject;
  className?: string;
};
export type InputType =
  | "text"
  | "number"
  | "password"
  | "email"
  | "checkbox"
  | "select";
export type OptionType = {
  value: string;
  label: string;
};
export type Input = {
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

export type Content = {
  paragraph?: Paragraph[];
  component?: AsyncComponent;
  input?: Input;
};

export type ActionReturn = Promise<boolean | void> | boolean;

export type Action<P> = {
  key: string;
  label: string;
  action?: ActionFn<P, ActionReturn>;
  actions?: Action<P>[];
  confirmation?: Content[] | ActionFn<P, Content[]>;
  confirmationHeader?: string | ActionFn<P, string>;
};
