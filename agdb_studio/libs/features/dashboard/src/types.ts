import type { ResultView } from "@agdb-studio/query/src/utils/resultShape";
import type { ColorConfig } from "@agdb-studio/query/src/utils/graphColorRules";

export const WIDGET_COLS = [1, 2, 3, 4] as const;
export const WIDGET_ROWS = [1, 2, 3] as const;

export type WidgetSize = {
  cols: (typeof WIDGET_COLS)[number];
  rows: (typeof WIDGET_ROWS)[number];
};

export const DEFAULT_WIDGET_SIZE: Readonly<WidgetSize> = { cols: 2, rows: 1 };

export const RESULT_VIEWS = [
  "graph",
  "table",
  "json",
  "scalar",
] as const satisfies readonly ResultView[];

/** One builder row: its query chain in the query URL format, e.g. `select().ids(1)`. */
export type WidgetRow = {
  /** Kept so colours set per row stay with the row. */
  id: string;
  query: string;
};

/**
 * The tabs a widget keeps; their queries only read. The results still offer
 * the query view's element actions (details panel edits, delete, add edge),
 * which change the widget's database, as on the query page.
 */
export const WIDGET_TABS = ["exec", "context"] as const;
export type WidgetTab = (typeof WIDGET_TABS)[number];

/** What a widget shows; where and how big it is lives in the dashboard `layout`. */
export type Widget = {
  id: string;
  title: string;
  owner: string;
  db: string;
  rows: Record<WidgetTab, WidgetRow[]>;
  /** The results view picked in the editor; unset lets the results pick one. */
  view?: ResultView;
  /** Graph colouring, as in the query view. */
  colors?: ColorConfig;
  /** Optional auto refresh interval, see `isRefreshInterval` in utils/refresh.ts. */
  refreshSeconds?: number;
};

/** A widget's place on the grid: `layout` order is the grid order. */
export type WidgetLayout = WidgetSize & { id: string };

/** The document stored as the server-side dashboard `config`. */
export type DashboardDocument = {
  version: 2;
  /** Widget content by widget id. */
  widgets: Record<string, Widget>;
  /** Grid order and sizes; every widget has exactly one entry. */
  layout: WidgetLayout[];
};
