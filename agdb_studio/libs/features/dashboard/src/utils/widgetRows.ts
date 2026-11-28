import type {
  PinnedDetail,
  Query,
  QueryRow,
} from "@agdb-studio/query/src/composables/types";
import { useQueryStore } from "@agdb-studio/query/src/composables/queryStore";
import {
  stepsToUrlValue,
  urlValueToSteps,
} from "@agdb-studio/query/src/utils/queryUrl";
import {
  copyColorConfig,
  hasColoring,
  type ColorConfig,
} from "@agdb-studio/query/src/utils/graphColorRules";
import { copyChartConfig } from "@agdb-studio/query/src/utils/chartConfig";
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

/** Plain copies of pinned details (they may hold reactive objects, which cannot be cloned). */
const copyPins = (pins: readonly PinnedDetail[]): PinnedDetail[] =>
  pins.map(({ id, offset }) =>
    offset ? { id, offset: { x: offset.x, y: offset.y } } : { id },
  );

/** Puts a widget's rows, colours, view and chart into a (new) query store entry. */
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

/** Writes how the widget shows its results (colours, view, pinned details, chart) into its query store entry. */
export const seedDisplay = (queryId: string, widget: Widget): void => {
  const queryStore = useQueryStore();
  const query = queryStore.getQuery(queryId);
  /* v8 ignore next -- @preserve: seeded together with the rows first */
  if (!query) return;
  query.value.colors = widget.colors
    ? copyColorConfig(widget.colors)
    : undefined;
  queryStore.setView(queryId, widget.view);
  queryStore.setPins(queryId, widget.pins ? copyPins(widget.pins) : []);
  queryStore.setChart(queryId, widget.chart);
};

/** The widget with the colours, view, pinned details and chart of a query store entry. */
export const withDisplay = (widget: Widget, query: Query): Widget => {
  const result: Widget = { ...widget };
  delete result.view;
  delete result.colors;
  delete result.pins;
  delete result.chart;
  if (query.view) result.view = query.view;
  if (query.chart) result.chart = copyChartConfig(query.chart);
  if (query.pins?.length) result.pins = copyPins(query.pins);
  if (query.colors && hasColoring(query.colors)) {
    result.colors = copyColorConfig(query.colors);
  }
  return result;
};

/** The widget as edited in a query store entry: rows, colours, view, pins and chart. */
export const widgetFromQuery = (base: Widget, query: Query): Widget =>
  withDisplay(
    {
      ...base,
      rows: {
        exec: fromQueryRows(query.rows.exec),
        context: fromQueryRows(query.rows.context),
      },
    },
    query,
  );

/** JSON with object keys sorted, so equal values give equal text; undefined stays undefined. */
const stableJson = (value: unknown): string | undefined =>
  JSON.stringify(value, (_key, item: unknown) =>
    typeof item === "object" && item !== null && !Array.isArray(item)
      ? Object.fromEntries(
          Object.entries(item).sort(([a], [b]) => (a < b ? -1 : 1)),
        )
      : item,
  );

/** Colours worth keeping: unset when nothing is coloured. */
const coloring = (colors: ColorConfig | undefined) =>
  colors && hasColoring(colors) ? colors : undefined;

/** Pins worth keeping: unset when there are none. */
const pinning = (pins: PinnedDetail[] | undefined) =>
  pins?.length ? pins : undefined;

/**
 * The entry shows the results differently (colours, view, pinned details or chart)
 * than the widget saves. Reads the entry in full (not a raw copy), so a
 * computed using this tracks every nested change.
 */
export const displayDiffers = (widget: Widget, query: Query): boolean =>
  (query.view ?? undefined) !== widget.view ||
  stableJson(coloring(query.colors)) !== stableJson(coloring(widget.colors)) ||
  stableJson(pinning(query.pins)) !== stableJson(pinning(widget.pins)) ||
  stableJson(query.chart) !== stableJson(widget.chart);

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
