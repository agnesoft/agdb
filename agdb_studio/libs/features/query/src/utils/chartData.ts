import type { DbElement, QueryResult } from "@agnesoft/agdb_api";
import { dbValueToString, dedupeResultElements } from "./graphDataUtils";
import { maxOf, minOf } from "./chartScale";
import {
  dbValueNumber,
  numericValue,
  type ChartAggregate,
  type ChartConfig,
} from "./chartConfig";

export type ChartPoint = {
  key: string;
  label: string;
  value: number;
  /** The folded groups past the limit, shown in a neutral colour. */
  other?: boolean;
};

/** How the groups of a line chart are ordered along X. */
export type ChartAxis = "number" | "date" | "category";

export type ChartData = {
  points: ChartPoint[];
  axis: ChartAxis;
  /** Elements left out: without the grouping key or a numeric measured value. */
  skipped: number;
  /** Groups folded into "Other". */
  folded: number;
};

/** Most groups shown before the rest fold into "Other". */
export const MAX_GROUPS: Record<ChartConfig["kind"], number> = {
  bar: 12,
  donut: 5,
  line: Infinity,
  stat: 1,
};

export const OTHER_LABEL = "Other";

type Group = {
  key: string;
  label: string;
  /** Elements (or, by query, the query's result number). */
  count: number;
  /** The numeric measured values. */
  nums: number[];
};

const fold = (
  aggregate: ChartAggregate,
  group: Pick<Group, "count" | "nums">,
): number | undefined => {
  if (aggregate === "count") return group.count;
  const { nums } = group;
  if (!nums.length) return undefined;
  switch (aggregate) {
    case "sum":
      return nums.reduce((a, b) => a + b, 0);
    case "avg":
      return nums.reduce((a, b) => a + b, 0) / nums.length;
    case "min":
      return minOf(nums);
    case "max":
      return maxOf(nums);
  }
};

const valueOf = (element: DbElement, key: string) =>
  element.values.find((kv) => dbValueToString(kv.key) === key)?.value;

const isDate = (label: string) =>
  /\d{4}-\d{2}/.test(label) && !Number.isNaN(Date.parse(label));

/** Orders line chart groups by number, then date, then natural string order. */
const axisOf = (labels: string[]): ChartAxis => {
  if (labels.every((l) => numericValue(l) !== undefined)) return "number";
  if (labels.every(isDate)) return "date";
  return "category";
};

type Labelled = { label: string };

const AXIS_ORDER: Record<ChartAxis, (a: Labelled, b: Labelled) => number> = {
  number: (a, b) => Number(a.label) - Number(b.label),
  date: (a, b) => Date.parse(a.label) - Date.parse(b.label),
  category: (a, b) =>
    a.label.localeCompare(b.label, undefined, { numeric: true }),
};

/**
 * The measured value of an element, if it has a number in the measured key.
 * Count measures nothing.
 */
const measuredValue = (
  element: DbElement,
  measure: ChartConfig["measure"],
): number | undefined => {
  if (measure.aggregate === "count") return undefined;
  const raw = valueOf(element, measure.key);
  return raw === undefined ? undefined : dbValueNumber(raw);
};

/** Elements grouped by their value of `key`; the rest are counted as skipped. */
const groupByProperty = (
  results: QueryResult[],
  key: string,
  measure: ChartConfig["measure"],
): { groups: Group[]; skipped: number } => {
  let skipped = 0;
  const byLabel = new Map<string, Group>();
  for (const { element } of dedupeResultElements(results)) {
    const raw = valueOf(element, key);
    if (raw === undefined) {
      skipped++;
      continue;
    }
    const label = dbValueToString(raw);
    const group = byLabel.get(label) ?? {
      key: label,
      label,
      count: 0,
      nums: [],
    };
    group.count++;
    const n = measuredValue(element, measure);
    if (n !== undefined) group.nums.push(n);
    else if (measure.aggregate !== "count") skipped++;
    byLabel.set(label, group);
  }
  return { groups: [...byLabel.values()], skipped };
};

/**
 * The points of a chart of query results. Grouped by query, group `i` is
 * result `i` (labelled `queryLabels[i]`) and its count is the query's result
 * number, so counting queries (e.g. `node_count`) chart too. Grouped by a
 * property, elements returned by several queries count once.
 */
export const computeChart = (
  results: QueryResult[],
  config: ChartConfig,
  queryLabels: readonly string[] = [],
): ChartData => {
  const { measure } = config;
  let skipped = 0;
  /** The measured numbers of elements, counting those without one as skipped. */
  const numsOf = (elements: DbElement[]) => {
    if (measure.aggregate === "count") return [];
    const nums = elements.flatMap((el) => {
      const n = measuredValue(el, measure);
      return n === undefined ? [] : [n];
    });
    skipped += elements.length - nums.length;
    return nums;
  };

  let groups: Group[];
  if (config.kind === "stat") {
    const elements = dedupeResultElements(results).map((d) => d.element);
    groups = [
      {
        key: "all",
        label: "All",
        // Each element once, as the other measures and charts by property
        // count them; a query whose number is not its elements (a count, such
        // as `edge_count`) adds that number, as it does grouped by query.
        count:
          dedupeResultElements(
            results.filter((r) => r.result === r.elements.length),
          ).length +
          results
            .filter((r) => r.result !== r.elements.length)
            .reduce((sum, r) => sum + r.result, 0),
        nums: numsOf(elements),
      },
    ];
  } else if (config.groupBy.by === "query") {
    groups = results.map((r, i) => ({
      key: `query-${i}`,
      label: queryLabels[i] ?? `#${i + 1}`,
      count: r.result,
      nums: numsOf(r.elements),
    }));
  } else {
    const grouped = groupByProperty(results, config.groupBy.key, measure);
    groups = grouped.groups;
    skipped = grouped.skipped;
  }

  const valued = groups.flatMap((group) => {
    const value = fold(measure.aggregate, group);
    return value === undefined ? [] : [{ group, value }];
  });

  let axis: ChartAxis = "category";
  const largestFirst = (
    a: (typeof valued)[number],
    b: (typeof valued)[number],
  ) => b.value - a.value || AXIS_ORDER.category(a.group, b.group);
  if (config.kind === "line") {
    if (config.groupBy.by === "property") {
      axis = axisOf(valued.map((v) => v.group.label));
      valued.sort((a, b) => AXIS_ORDER[axis](a.group, b.group));
    }
  } else if (
    config.groupBy.by === "property" ||
    valued.length > MAX_GROUPS[config.kind]
  ) {
    // Largest first; queries keep batch order unless some fold into "Other",
    // which then holds the smallest.
    valued.sort(largestFirst);
  }

  const max = MAX_GROUPS[config.kind];
  const toPoint = ({ group, value }: (typeof valued)[number]): ChartPoint => ({
    key: group.key,
    label: group.label,
    value,
  });
  if (valued.length <= max) {
    return { points: valued.map(toPoint), axis, skipped, folded: 0 };
  }
  const shown = valued.slice(0, max - 1);
  const rest = valued.slice(max - 1);
  const other = fold(measure.aggregate, {
    count: rest.reduce((sum, v) => sum + v.group.count, 0),
    nums: rest.flatMap((v) => v.group.nums),
  });
  return {
    points: [
      ...shown.map(toPoint),
      // Folded groups all have a value, so their fold has one too.
      { key: "other", label: OTHER_LABEL, value: other ?? 0, other: true },
    ],
    axis,
    skipped,
    folded: rest.length,
  };
};

/** A line of a chart split by query: its value at each X, if it has one. */
export type ChartSeries = {
  key: string;
  label: string;
  /** A name short enough to sit at the line's end, e.g. "#2". */
  short: string;
  /** The palette slot; "Other" has none and is drawn neutral. */
  slot?: number;
  values: (number | undefined)[];
};

export type ChartSeriesData = {
  /** The groups along X, shared by every series. */
  xs: { key: string; label: string }[];
  series: ChartSeries[];
  axis: ChartAxis;
  /** Elements left out, counted once per query that returned them. */
  skipped: number;
  /** Queries folded into the "Other" series. */
  folded: number;
};

/** Most series drawn: one per palette colour, the last as "Other" if needed. */
export const MAX_SERIES = 5;

/**
 * The series of a chart split by query: each query's elements are grouped by
 * the property along X and measured, as a chart of that query alone would.
 * Queries keep batch order (and so their colour); past `MAX_SERIES` the rest
 * fold into one "Other" series. A query without a group at some X has a gap
 * there, not a zero.
 */
export const computeSeries = (
  results: QueryResult[],
  key: string,
  measure: ChartConfig["measure"],
  queryLabels: readonly string[] = [],
): ChartSeriesData => {
  let skipped = 0;
  type Line = {
    key: string;
    label: string;
    short: string;
    slot?: number;
    groups: Group[];
  };
  const grouped = results.map((r, i): Line => {
    const g = groupByProperty([r], key, measure);
    skipped += g.skipped;
    return {
      key: `query-${i}`,
      label: queryLabels[i] ?? `#${i + 1}`,
      short: `#${i + 1}`,
      slot: i,
      groups: g.groups,
    };
  });
  const lines =
    grouped.length <= MAX_SERIES ? grouped : grouped.slice(0, MAX_SERIES - 1);
  const rest = grouped.slice(lines.length);
  if (rest.length) {
    const merged = new Map<string, Group>();
    for (const { groups } of rest) {
      for (const group of groups) {
        const into = merged.get(group.label) ?? {
          ...group,
          count: 0,
          nums: [],
        };
        into.count += group.count;
        into.nums.push(...group.nums);
        merged.set(group.label, into);
      }
    }
    lines.push({
      key: "other",
      label: OTHER_LABEL,
      short: OTHER_LABEL,
      groups: [...merged.values()],
    });
  }

  const valued = lines.map((line) => {
    const values = new Map<string, number>();
    for (const group of line.groups) {
      const value = fold(measure.aggregate, group);
      if (value !== undefined) values.set(group.label, value);
    }
    return { line, values };
  });
  const labels = [
    ...new Set(valued.flatMap(({ values }) => [...values.keys()])),
  ];
  const axis = axisOf(labels);
  const xs = labels
    .map((label) => ({ key: label, label }))
    .sort(AXIS_ORDER[axis]);
  return {
    xs,
    series: valued.map(({ line, values }) => ({
      key: line.key,
      label: line.label,
      short: line.short,
      ...(line.slot === undefined ? {} : { slot: line.slot }),
      values: xs.map((x) => values.get(x.label)),
    })),
    axis,
    skipped,
    folded: rest.length,
  };
};
