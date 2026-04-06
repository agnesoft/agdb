import { queryApiMock } from "../mock/queryApiMock";

export type QueryType = keyof typeof queryApiMock;

/** One field's user input: the selected type option and an optional literal value. */
export type QueryStepFieldValue = {
  selectedOption: string;
  value?: string;
};

/** One "row" of arguments — one `QueryStepFieldValue` per field in the schema. */
export type QueryStepArgEntry = QueryStepFieldValue[];

export type QueryStep = {
  id: string;
  type: QueryType;
  args?: QueryStepArgEntry[];
  invalid?: boolean;
};

export type AddQueryParams = {
  id: string;
  name?: string;
};

export type Query = {
  isRunning: boolean;
  lastRun?: Date;
  steps: {
    exec: QueryStep[];
    exec_mut: QueryStep[];
    context: QueryStep[];
  };
} & AddQueryParams;

export const TABS = ["exec", "exec_mut", "context"] as const;

export type TAB = (typeof TABS)[number];
