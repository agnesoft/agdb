import type { Components } from "@agnesoft/agdb_api";

export type ResultView = "graph" | "table" | "json" | "scalar";

export type ResultSummary = {
  index: number;
  result: number;
  nodes: number;
  edges: number;
  /** Elements that are neither nodes nor edges (id 0, e.g. `indexes`). */
  other: number;
};

/** Query steps whose results describe metadata rather than graph structure. */
export const TABLE_STEP_TYPES = new Set([
  "select.keys",
  "select.key_count",
  "select.edge_count",
  "select.edge_count_from",
  "select.edge_count_to",
  "select.aliases",
  "select.indexes",
]);

const SCALAR_LABELS: Record<string, string> = {
  "select.node_count": "Node count",
};

export const summarizeResults = (
  results: Components.Schemas.QueryResult[],
): ResultSummary[] =>
  results.map((r, index) => ({
    index,
    result: r.result,
    nodes: r.elements.filter((el) => el.id > 0).length,
    edges: r.elements.filter((el) => el.id < 0).length,
    other: r.elements.filter((el) => el.id === 0).length,
  }));

/** Picks the most suitable default view for the results of a query. */
export const classifyResults = (
  results: Components.Schemas.QueryResult[],
  stepTypes: string[],
): Exclude<ResultView, "json"> => {
  const elements = results.flatMap((r) => r.elements);
  if (elements.length === 0) return "scalar";
  if (stepTypes.some((type) => TABLE_STEP_TYPES.has(type))) return "table";
  if (elements.every((el) => el.id === 0)) return "table";
  return "graph";
};

/** Label for the scalar card, e.g. "Node count" or "Affected". */
export const scalarLabel = (stepTypes: string[], mutation: boolean): string => {
  for (const type of stepTypes) {
    const label = SCALAR_LABELS[type];
    if (label) return label;
  }
  return mutation ? "Affected" : "Result";
};
