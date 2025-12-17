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
  steps: QueryStep[];
};

export type Query = {
  isRunning: boolean;
  lastRun?: Date;
} & AddQueryParams;
