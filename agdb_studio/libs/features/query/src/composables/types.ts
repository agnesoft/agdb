import { queryApiMock } from "../mock/queryApiMock";

export type QueryType = keyof typeof queryApiMock;

export type QueryStep = {
  id: string;
  type: QueryType;
  values?: string[];
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
