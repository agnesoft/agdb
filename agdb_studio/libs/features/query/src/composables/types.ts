import type { ColorConfig } from "../utils/graphColorRules";
import type { ResultView } from "../utils/resultShape";
import type { ChartConfig } from "../utils/chartConfig";
import type {
  DbKeyValue,
  QueryResult,
  QueryType as AgdbQueryType,
} from "@agnesoft/agdb_api";
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
  queryOnlyValues?: DbKeyValue[];
  /** The node's alias, if it has one. */
  alias?: string;
  /** Indexes of the main results (queries of the batch) that returned the element. */
  resultIndexes?: number[];
};

/** A single node in the graph visualization. */
export type GraphNode = {
  id: number;
  values: { key: string; value: string }[];
  /** Original typed key-values, kept so they can be edited without losing types. */
  rawValues: DbKeyValue[];
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
  rawValues: DbKeyValue[];
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
  exec?: QueryResult[];
  exec_mut?: QueryResult[];
  context?: QueryResult[];
  /** Elements loaded on demand from the graph (e.g. "Expand neighbours"). */
  expanded?: QueryResult[];
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
  executedQueries?: Partial<Record<TAB, AgdbQueryType[]>>;
  /** The step types of those queries, which decide how their results are shown. */
  executedStepTypes?: Partial<Record<TAB, QueryType[]>>;
  /**
   * The ids of the rows the last run of each tab sent, in order: result `i`
   * came from row `i`. Rows moved, added or removed since don't change it.
   */
  executedRowIds?: Partial<Record<TAB, string[]>>;
  /**
   * The exec_mut rows were opened from a link and have not been confirmed
   * yet, so running them asks first (a link could hold a harmful query).
   */
  unconfirmedLinkedMutation?: boolean;
  rows: {
    exec: QueryRow[];
    exec_mut: QueryRow[];
    context: QueryRow[];
  };
  results: QueryResults;
  /** How the graph colours elements (rules, per row, per element). */
  colors?: ColorConfig;
  /** The results view the user picked; unset lets the results pick one. */
  view?: ResultView;
  /** Elements whose details stay shown on the graph, in pinning order. */
  pins?: PinnedDetail[];
  /** What the chart view shows; unset until the chart view is first picked. */
  chart?: ChartConfig;
} & AddQueryParams;

/** A point on screen, in px. */
export type ScreenPoint = { x: number; y: number };

/**
 * An element whose details card stays shown on the graph. `offset` is where
 * the card was dragged to, relative to its element; unset is the default spot.
 */
export type PinnedDetail = { id: number; offset?: ScreenPoint };

export const TABS = ["exec", "exec_mut", "context"] as const;

export type TAB = (typeof TABS)[number];
