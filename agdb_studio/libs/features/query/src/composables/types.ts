export type QueryType = "select" | "insert" | "update" | "delete" | "raw";

export type QueryStep = {
  id: string;
  type: QueryType;
  name: string;
  data?: string;
};

export type AddQueryParams = {
  id: string;
  name: string;
  steps: {
    exec: QueryStep[];
    exec_mut: QueryStep[];
    context: QueryStep[];
  };
};

export type Query = {
  isRunning: boolean;
  lastRun?: Date;
} & AddQueryParams;

export const TABS = ["exec", "exec_mut", "context"] as const;

export type TAB = (typeof TABS)[number];
