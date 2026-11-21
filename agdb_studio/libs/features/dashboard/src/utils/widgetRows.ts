import { toRaw } from "vue";
import type { Query, QueryRow } from "@agdb-studio/query/src/composables/types";
import { useQueryStore } from "@agdb-studio/query/src/composables/queryStore";
import {
  stepsToUrlValue,
  urlValueToSteps,
} from "@agdb-studio/query/src/utils/queryUrl";
import { hasColoring } from "@agdb-studio/query/src/utils/graphColorRules";
import { buildQueryFromSteps } from "@agdb-studio/query/src/utils/queryExecutionUtils";
import { WIDGET_TABS, type Widget, type WidgetRow } from "../types";

const emptyRow = (): QueryRow => ({
  id: `row-${crypto.randomUUID()}`,
  steps: [],
});

/** Builder rows for a widget's rows; row ids are kept, unreadable rows skipped. */
export const toQueryRows = (rows: WidgetRow[]): QueryRow[] => {
  const converted = rows.flatMap((row) => {
    const steps = urlValueToSteps(row.query);
    return steps
      ? [
          {
            id: row.id,
            steps: steps.map((step, i) => ({ ...step, id: `${row.id}-${i}` })),
          },
        ]
      : [];
  });
  // A tab always has a row to type into, as in the query view.
  return converted.length ? converted : [emptyRow()];
};

/** Widget rows from builder rows; empty rows are left out. */
export const fromQueryRows = (rows: QueryRow[]): WidgetRow[] =>
  rows
    .filter((row) => row.steps.length)
    .map((row) => ({ id: row.id, query: stepsToUrlValue(row.steps) }));

/** Puts a widget's rows, colours and view into a (new) query store entry. */
export const seedQuery = (queryId: string, widget: Widget): void => {
  const queryStore = useQueryStore();
  const query =
    queryStore.getQuery(queryId) ?? queryStore.addQuery({ id: queryId });
  for (const tab of WIDGET_TABS) {
    query.value.rows[tab] = toQueryRows(widget.rows[tab]);
    queryStore.validateQuerySteps(queryId, tab);
  }
  seedDisplay(queryId, widget);
};

/** Writes how the widget shows its results (colours and view) into its query store entry. */
export const seedDisplay = (queryId: string, widget: Widget): void => {
  const queryStore = useQueryStore();
  const query = queryStore.getQuery(queryId);
  /* v8 ignore next -- @preserve: seeded together with the rows first */
  if (!query) return;
  query.value.colors = widget.colors
    ? structuredClone(toRaw(widget.colors))
    : undefined;
  queryStore.setView(queryId, widget.view);
};

/** The widget as edited in a query store entry: rows, colours and view. */
export const widgetFromQuery = (base: Widget, query: Query): Widget => {
  const widget: Widget = {
    ...base,
    rows: {
      exec: fromQueryRows(query.rows.exec),
      context: fromQueryRows(query.rows.context),
    },
  };
  delete widget.view;
  delete widget.colors;
  if (query.view) widget.view = query.view;
  if (query.colors && hasColoring(query.colors)) {
    widget.colors = structuredClone(toRaw(query.colors));
  }
  return widget;
};

/** A row of the exec or context tab is not a complete, valid query yet. */
export const hasUnfinishedRow = (query: Query): boolean =>
  WIDGET_TABS.some((tab) =>
    query.rows[tab].some((row) => {
      if (!row.steps.length) return false;
      try {
        buildQueryFromSteps(row.steps);
        return false;
      } catch {
        return true;
      }
    }),
  );
