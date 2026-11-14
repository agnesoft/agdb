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

/** A step as it is copied: no id (a new one is made on paste) and no validation state. */
export type CopiedStep = Pick<QueryStep, "type" | "args">;

/** One independent query (a chain of steps). A tab holds several rows that run as one transaction. */
export type QueryRow = {
  id: string;
  steps: QueryStep[];
};

export type AddQueryParams = {
  id: string;
  name?: string;
};

/** Distinguishes main query results from context (supplementary) results in the graph. */
export type GraphElementSource = "query" | "context";

/**
 * Loading state of an element's stored values (all values in the database,
 * not only those the query returned).
 */
export type ElementValuesState = "loading" | "loaded" | "error";

/** Stored-values details added to elements shown in the tooltip and panel. */
export type StoredValuesInfo = {
  /** When "loaded", `values`/`rawValues` hold all stored values. */
  valuesState?: ElementValuesState;
  /** Values the query returned that are not stored (e.g. a computed `edge_count`). */
  queryOnlyValues?: Components.Schemas.DbKeyValue[];
};

/** A single node in the graph visualization. */
export type GraphNode = {
  id: number;
  values: { key: string; value: string }[];
  /** Original typed key-values, kept so they can be edited without losing types. */
  rawValues: Components.Schemas.DbKeyValue[];
  source: GraphElementSource;
} & StoredValuesInfo;

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
  /** Original typed key-values, kept so they can be edited without losing types. */
  rawValues: Components.Schemas.DbKeyValue[];
  source: number;
  target: number;
  elementSource: GraphElementSource;
} & StoredValuesInfo;

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
  /** Elements loaded on demand from the graph (e.g. "Expand neighbours"). */
  expanded?: Components.Schemas.QueryResult[];
};

/** A graph element (node or edge) that user actions operate on. */
export type GraphElement = GraphNode | GraphLink;

export type Query = {
  isRunning: boolean;
  lastRun?: Date;
  lastExecutedTab?: "exec" | "exec_mut";
  /**
   * The exact queries sent by the last run of each tab. Graph actions re-run
   * these after a change, so edits in the builder that were never run are not
   * picked up.
   */
  executedQueries?: Partial<Record<TAB, Components.Schemas.QueryType[]>>;
  /** The step types of those queries, which decide how their results are shown. */
  executedStepTypes?: Partial<Record<TAB, QueryType[]>>;
  rows: {
    exec: QueryRow[];
    exec_mut: QueryRow[];
    context: QueryRow[];
  };
  results: QueryResults;
} & AddQueryParams;

export const TABS = ["exec", "exec_mut", "context"] as const;

export type TAB = (typeof TABS)[number];
