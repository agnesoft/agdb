import type { Components } from "@agnesoft/agdb_api";
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

/** Distinguishes main query results from context (supplementary) results in the graph. */
export type GraphElementSource = "query" | "context";

/** A single node in the graph visualization. */
export type GraphNode = {
  id: number;
  values: { key: string; value: string }[];
  source: GraphElementSource;
};

/**
 * A single link/edge in the graph visualization.
 *
 * NOTE: `source` and `target` are declared as numbers (node ids) to match our
 * input data, but 3d-force-graph mutates them into full node objects at runtime.
 * This is why QueryGraph uses `as unknown as GraphLink` when casting from the
 * library's generic link type.
 */
export type GraphLink = {
  id: number;
  values: { key: string; value: string }[];
  source: number;
  target: number;
  elementSource: GraphElementSource;
};

/** The complete graph data structure consumed by 3d-force-graph. */
export type GraphData = {
  nodes: GraphNode[];
  links: GraphLink[];
};

/** Raw API results stored per tab. */
export type QueryResults = {
  exec?: Components.Schemas.QueryResult[];
  exec_mut?: Components.Schemas.QueryResult[];
  context?: Components.Schemas.QueryResult[];
};

export type Query = {
  isRunning: boolean;
  lastRun?: Date;
  lastExecutedTab?: "exec" | "exec_mut";
  steps: {
    exec: QueryStep[];
    exec_mut: QueryStep[];
    context: QueryStep[];
  };
  results: QueryResults;
} & AddQueryParams;

export const TABS = ["exec", "exec_mut", "context"] as const;

export type TAB = (typeof TABS)[number];
