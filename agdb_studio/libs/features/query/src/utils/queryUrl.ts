import type { CopiedStep, QueryRow, TAB } from "../composables/types";
import { TABS } from "../composables/types";
import { formatSteps, parseSteps } from "./queryRowClipboard";

const PREFIX = "QueryBuilder::";
const SUFFIX = ".query()";

/** The builder rows of every tab as URL query params: one value per row. */
export type QueryUrlParams = Partial<Record<TAB, string[]>>;

/** Rows read back from the URL, per tab, and how many values could not be read. */
export type QueryUrlRows = {
  rows: Partial<Record<TAB, CopiedStep[][]>>;
  skipped: Partial<Record<TAB, number>>;
};

/**
 * Formats steps as a short query chain for the URL, e.g. `select().ids(1)`:
 * the copied Rust code without the `QueryBuilder::` and `.query()` around it.
 */
export const stepsToUrlValue = (steps: CopiedStep[]): string => {
  const code = formatSteps(steps);
  // The formatter always adds both.
  /* v8 ignore next -- @preserve */
  const start = code.startsWith(PREFIX) ? PREFIX.length : 0;
  /* v8 ignore next -- @preserve */
  const end = code.endsWith(SUFFIX) ? code.length - SUFFIX.length : undefined;
  return code.slice(start, end);
};

/** Reads steps from a URL value. Values that are not a query give `undefined`. */
export const urlValueToSteps = (value: string): CopiedStep[] | undefined => {
  const result = parseSteps(value);
  return "steps" in result ? result.steps : undefined;
};

/** Rows with steps become URL params; empty rows and tabs are left out. */
export const rowsToUrlParams = (
  rows: Record<TAB, QueryRow[]>,
): QueryUrlParams => {
  const params: QueryUrlParams = {};
  for (const tab of TABS) {
    const values = rows[tab]
      .filter((row) => row.steps.length)
      .map((row) => stepsToUrlValue(row.steps));
    if (values.length) params[tab] = values;
  }
  return params;
};

const toValues = (param: unknown): string[] => {
  const values: unknown[] = Array.isArray(param) ? param : [param];
  return values.filter((value): value is string => typeof value === "string");
};

/** Reads rows from route query params, counting the values that cannot be read. */
export const urlParamsToRows = (
  query: Record<string, unknown>,
): QueryUrlRows => {
  const result: QueryUrlRows = { rows: {}, skipped: {} };
  for (const tab of TABS) {
    const values = toValues(query[tab]);
    const tabRows = values
      .map(urlValueToSteps)
      .filter((steps): steps is CopiedStep[] => !!steps);
    if (tabRows.length) result.rows[tab] = tabRows;
    if (tabRows.length < values.length) {
      result.skipped[tab] = values.length - tabRows.length;
    }
  }
  return result;
};

/** Whether the route query already holds these params, so the URL needs no update. */
export const sameUrlParams = (
  query: Record<string, unknown>,
  params: QueryUrlParams,
): boolean =>
  TABS.every((tab) => {
    const current = toValues(query[tab]);
    const next = params[tab] ?? [];
    return (
      current.length === next.length &&
      current.every((value, i) => value === next[i])
    );
  });

/**
 * Longest query string the rows may use, as measured by `urlParamsLength`.
 * A reload sends the whole URL to the server, and servers and proxies often
 * reject request lines over 8 KB (e.g. nginx's default buffer), so some room
 * is left for the rest.
 */
export const MAX_QUERY_URL_LENGTH = 6000;

/**
 * Length of the rows in a query string, e.g. `exec=select().ids(1)&…`. This
 * is an upper bound: it encodes every value fully, while the router leaves
 * some characters (e.g. `,` and `[`) as they are.
 */
export const urlParamsLength = (query: Record<string, unknown>): number =>
  TABS.reduce(
    (total, tab) =>
      toValues(query[tab]).reduce(
        (sum, value) => sum + tab.length + encodeURIComponent(value).length + 2,
        total,
      ),
    0,
  );
