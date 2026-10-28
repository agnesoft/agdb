import type { Component } from "vue";
import type { StyleObject } from "@agdb-studio/design/src/types/base";

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
  "text" | "number" | "password" | "email" | "checkbox" | "select";
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

export type RowAction = {
  label: string;
  icon?: Component;
  onClick: (rowIndex: number) => void | Promise<void>;
};

export type ContentTable = {
  headers: string[];
  rows: string[][];
  columnClasses?: (string | undefined)[];
  rowActions?: RowAction[];
};

export type Content = {
  paragraph?: Paragraph[];
  input?: Input;
  table?: ContentTable;
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
