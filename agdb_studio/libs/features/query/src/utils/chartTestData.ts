import type { DbElement, DbValue, QueryResult } from "@agnesoft/agdb_api";

/** Test data for the chart specs: elements with string and integer values. */
export const chartElement = (
  id: number,
  values: Record<string, string | number>,
): DbElement => ({
  id,
  from: 0,
  to: 0,
  values: Object.entries(values).map(([key, value]) => {
    const dbValue: DbValue =
      typeof value === "number" ? { I64: value } : { String: value };
    return { key: { String: key }, value: dbValue };
  }),
});

/** A query result of `elements`; `count` is its result number (default: how many). */
export const chartResult = (
  elements: DbElement[],
  count = elements.length,
): QueryResult => ({ result: count, elements });
