export type QueryType = "select" | "insert" | "update" | "delete" | "raw";

export type QueryStep = {
  id: string;
  type: QueryType;
  name: string;
  data?: string;
};

export type Query = {
  id: string;
  name: string;
  steps: QueryStep[];
  isRunning: boolean;
  lastRun?: Date;
};
