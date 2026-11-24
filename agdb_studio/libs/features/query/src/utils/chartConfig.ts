import type { QueryResult } from "@agnesoft/agdb_api";
import { convertToNativeValue } from "@agnesoft/agdb_api";
import { dbValueToString, dedupeResultElements } from "./graphDataUtils";

export const CHART_KINDS = ["bar", "line", "donut", "stat"] as const;
export type ChartKind = (typeof CHART_KINDS)[number];

export const CHART_AGGREGATES = ["count", "sum", "avg", "min", "max"] as const;
export type ChartAggregate = (typeof CHART_AGGREGATES)[number];

/** What each bar, point or slice stands for. */
export type ChartGroupBy = { by: "property"; key: string } | { by: "query" };

/** What a group's value is: how many elements, or a numeric property folded. */
export type ChartMeasure =
  | { aggregate: "count" }
  | { aggregate: Exclude<ChartAggregate, "count">; key: string };

/** What the chart draws a series of its own for: each query of the batch. */
export type ChartSplit = { by: "query" };

/**
 * A chart of query results: elements are grouped (by a property's values or
 * by the query of the batch that returned them) and each group measured. A
 * stat tile measures all elements as one group. A line grouped by a property
 * may be `split` into a line per query.
 */
export type ChartConfig = {
  kind: ChartKind;
  groupBy: ChartGroupBy;
  measure: ChartMeasure;
  /** Kept while the chart is another kind, which ignores it. */
  split?: ChartSplit;
};

/** Whether the chart draws a series per query. */
export const isSplit = (config: ChartConfig): boolean =>
  config.kind === "line" &&
  config.groupBy.by === "property" &&
  config.split?.by === "query";

export const AGGREGATE_LABELS: Record<ChartAggregate, string> = {
  count: "Count",
  sum: "Sum",
  avg: "Average",
  min: "Min",
  max: "Max",
};

export const measureLabel = (measure: ChartMeasure): string =>
  measure.aggregate === "count"
    ? "Count"
    : `${AGGREGATE_LABELS[measure.aggregate]} of ${measure.key}`;

/**
 * e.g. "Average of price by type", "Count by query", "Sum of price",
 * "Count by day, per query".
 */
export const chartLabel = (config: ChartConfig): string => {
  const measure = measureLabel(config.measure);
  if (config.kind === "stat") return measure;
  const label = `${measure} by ${config.groupBy.by === "query" ? "query" : config.groupBy.key}`;
  return isSplit(config) ? `${label}, per query` : label;
};

/** The number in a value, if it is one (numbers, and strings that read as one). */
export const numericValue = (native: unknown): number | undefined => {
  if (typeof native === "number")
    return Number.isFinite(native) ? native : undefined;
  if (typeof native === "string" && native.trim() !== "") {
    const n = Number(native);
    return Number.isFinite(n) ? n : undefined;
  }
  return undefined;
};

export type ResultKey = {
  key: string;
  /** Elements that have the key. */
  count: number;
  /**
   * Some value of the key is a number, so it can be measured; elements
   * without one are left out of the measure (and say so).
   */
  numeric: boolean;
  /** Some value of the key is not a number, so it can name a group. */
  textual: boolean;
  /** Distinct values (as shown). */
  distinct: number;
};

/** The property keys of the results' elements, in order of first appearance. */
export const resultKeys = (results: QueryResult[]): ResultKey[] => {
  const keys = new Map<
    string,
    { count: number; numeric: boolean; textual: boolean; values: Set<string> }
  >();
  for (const { element } of dedupeResultElements(results)) {
    for (const kv of element.values) {
      const key = dbValueToString(kv.key);
      const entry = keys.get(key) ?? {
        count: 0,
        numeric: false,
        textual: false,
        values: new Set<string>(),
      };
      entry.count++;
      entry.values.add(dbValueToString(kv.value));
      if (numericValue(convertToNativeValue(kv.value)) === undefined) {
        entry.textual = true;
      } else entry.numeric = true;
      keys.set(key, entry);
    }
  }
  return [...keys.entries()].map(
    ([key, { count, numeric, textual, values }]) => ({
      key,
      count,
      numeric,
      textual,
      distinct: values.size,
    }),
  );
};

/** Most groups a suggested chart starts with; more read as noise. */
const SUGGEST_MAX_GROUPS = 12;

/**
 * A chart that fits the results, so picking "Chart" shows something useful
 * at once: one bar per query for a batch of counts, a stat tile for a single
 * count, else elements counted by their first property with a few values.
 */
export const suggestChart = (results: QueryResult[]): ChartConfig => {
  const count: ChartMeasure = { aggregate: "count" };
  const byQuery: ChartGroupBy = { by: "query" };
  const hasElements = results.some((r) => r.elements.length > 0);
  if (!hasElements) {
    return {
      kind: results.length > 1 ? "bar" : "stat",
      groupBy: byQuery,
      measure: count,
    };
  }
  // Some value must be shared: a key unique to each element (e.g. a name)
  // would only draw a bar of 1 per element.
  const category = resultKeys(results).find(
    (k) =>
      k.textual &&
      k.distinct >= 2 &&
      k.distinct < k.count &&
      k.distinct <= SUGGEST_MAX_GROUPS,
  );
  if (category) {
    return {
      kind: "bar",
      groupBy: { by: "property", key: category.key },
      measure: count,
    };
  }
  return {
    kind: results.length > 1 ? "bar" : "stat",
    groupBy: byQuery,
    measure: count,
  };
};

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

const isKey = (value: unknown): value is string =>
  typeof value === "string" && value !== "";

/** Reads a stored chart config; anything malformed gives `undefined`. */
export const readChartConfig = (value: unknown): ChartConfig | undefined => {
  if (!isRecord(value)) return undefined;
  const { groupBy: group, measure: stored } = value;
  if (!isRecord(group) || !isRecord(stored)) return undefined;
  const kind = CHART_KINDS.find((k) => k === value.kind);
  const aggregate = CHART_AGGREGATES.find((a) => a === stored.aggregate);
  if (!kind || !aggregate) return undefined;

  let groupBy: ChartGroupBy;
  if (group.by === "query") groupBy = { by: "query" };
  else if (group.by === "property" && isKey(group.key)) {
    groupBy = { by: "property", key: group.key };
  } else return undefined;

  let measure: ChartMeasure;
  if (aggregate === "count") measure = { aggregate };
  else if (isKey(stored.key)) measure = { aggregate, key: stored.key };
  else return undefined;

  // An unknown split is left out rather than losing the whole chart.
  const split = isRecord(value.split) && value.split.by === "query";
  return {
    kind,
    groupBy,
    measure,
    ...(split ? { split: { by: "query" } } : {}),
  };
};

/** Aggregates whose groups add up to a whole, as a donut shows them. */
export const ADDITIVE_AGGREGATES: readonly ChartAggregate[] = ["count", "sum"];

/**
 * The config as it can be shown: a donut shows shares of a total, so an
 * average, min or max (which do not add up) becomes a sum of the same key;
 * a line runs along an ordered X, which the queries of a batch are not, so
 * a line by query becomes bars.
 */
export const showableChart = (config: ChartConfig): ChartConfig => {
  const { kind, measure } = config;
  if (kind === "line" && config.groupBy.by === "query") {
    return { ...config, kind: "bar" };
  }
  if (kind !== "donut" || measure.aggregate === "count") return config;
  if (ADDITIVE_AGGREGATES.includes(measure.aggregate)) return config;
  return { ...config, measure: { aggregate: "sum", key: measure.key } };
};

/** A plain copy (stored configs may be reactive, which cannot be cloned). */
export const copyChartConfig = (config: ChartConfig): ChartConfig => ({
  kind: config.kind,
  groupBy: { ...config.groupBy },
  measure: { ...config.measure },
  ...(config.split ? { split: { ...config.split } } : {}),
});
