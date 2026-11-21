import type { PinnedDetail } from "@agdb-studio/query/src/composables/types";
import type {
  ColorConfig,
  ColorRule,
  PaletteSlot,
} from "@agdb-studio/query/src/utils/graphColorRules";
import {
  DEFAULT_WIDGET_SIZE,
  RESULT_VIEWS,
  WIDGET_COLS,
  WIDGET_ROWS,
  type DashboardDocument,
  type Widget,
  type WidgetLayout,
  type WidgetRow,
  type WidgetSize,
} from "../types";
import { isRefreshInterval } from "./refresh";

/** The document version this Studio writes; newer ones are not read. */
export const DOCUMENT_VERSION = 2;

/** A dashboard saved by a newer Studio: reading it would drop what it added. */
export class NewerDashboardError extends Error {
  constructor(version: number) {
    super(
      `This dashboard was saved by a newer agdb Studio (version ${version}). Update Studio to open it.`,
    );
    this.name = "NewerDashboardError";
  }
}

export const emptyDocument = (): DashboardDocument => ({
  version: 2,
  widgets: {},
  layout: [],
});

export const createWidget = (
  init: Pick<Widget, "owner" | "db"> & Partial<Widget>,
): Widget => ({
  id: `widget-${crypto.randomUUID()}`,
  title: "New widget",
  rows: { exec: [], context: [] },
  ...init,
});

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

export const oneOf = <T extends string | number>(
  options: readonly T[],
  value: unknown,
  fallback: T,
): T => options.find((option) => option === value) ?? fallback;

/**
 * Ids read from a document end up as object keys (widgets, row colours), where
 * a plain assignment of `__proto__` would replace the object's prototype.
 */
const isSafeKey = (key: string): boolean => key !== "__proto__";

const isSlot = (value: unknown): value is PaletteSlot =>
  typeof value === "number" && Number.isInteger(value) && value >= 0;

const readRows = (value: unknown): WidgetRow[] =>
  Array.isArray(value)
    ? value.flatMap((row) =>
        isRecord(row) &&
        typeof row.id === "string" &&
        isSafeKey(row.id) &&
        typeof row.query === "string"
          ? [{ id: row.id, query: row.query }]
          : [],
      )
    : [];

const readRule = (value: unknown): ColorRule[] => {
  if (
    !isRecord(value) ||
    typeof value.id !== "string" ||
    typeof value.key !== "string"
  ) {
    return [];
  }
  const { id, key } = value;
  if (value.kind === "palette") return [{ kind: "palette", id, key }];
  if (value.kind !== "property" || !isSlot(value.color)) return [];
  return [
    {
      kind: "property",
      id,
      key,
      op: value.op === "exists" ? "exists" : "eq",
      color: value.color,
      ...(typeof value.value === "string" ? { value: value.value } : {}),
    },
  ];
};

const readSlots = (value: unknown): Record<string, PaletteSlot> =>
  isRecord(value)
    ? Object.fromEntries(
        Object.entries(value).filter(
          (entry): entry is [string, PaletteSlot] =>
            isSafeKey(entry[0]) && isSlot(entry[1]),
        ),
      )
    : {};

/** Reads stored colours, keeping only well-formed rules and slots. */
const readColors = (value: unknown): ColorConfig | undefined => {
  if (!isRecord(value)) return undefined;
  const elements: Record<number, PaletteSlot> = {};
  for (const [id, slot] of Object.entries(readSlots(value.elements))) {
    if (Number.isInteger(Number(id))) elements[Number(id)] = slot;
  }
  return {
    rules: Array.isArray(value.rules) ? value.rules.flatMap(readRule) : [],
    rows: readSlots(value.rows),
    elements,
  };
};

const isFiniteNumber = (value: unknown): value is number =>
  typeof value === "number" && Number.isFinite(value);

/** Reads pinned details: element ids (edges are negative) once each, with a card offset. */
const readPins = (value: unknown): PinnedDetail[] => {
  if (!Array.isArray(value)) return [];
  const pins: PinnedDetail[] = [];
  for (const pin of value) {
    if (
      !isRecord(pin) ||
      !Number.isInteger(pin.id) ||
      pin.id === 0 ||
      pins.some((p) => p.id === pin.id)
    ) {
      continue;
    }
    const { offset } = pin;
    pins.push({
      id: Number(pin.id),
      ...(isRecord(offset) &&
      isFiniteNumber(offset.x) &&
      isFiniteNumber(offset.y)
        ? { offset: { x: offset.x, y: offset.y } }
        : {}),
    });
  }
  return pins;
};

/** Version 1 widgets held one query chain and a fixed view; charts are gone. */
const V1_VIEWS: Record<string, Widget["view"]> = {
  stat: "scalar",
  table: "table",
  graph: "graph",
  chart: "table",
};

/** Reads one widget, filling defaults; values that are not a widget give `undefined`. */
export const readWidget = (value: unknown): Widget | undefined => {
  if (!isRecord(value)) return undefined;
  const { id, title, owner, db } = value;
  if (
    typeof id !== "string" ||
    !isSafeKey(id) ||
    typeof owner !== "string" ||
    typeof db !== "string"
  ) {
    return undefined;
  }
  const rows = isRecord(value.rows) ? value.rows : {};
  const widgetRows: Widget["rows"] = {
    exec: readRows(rows.exec),
    context: readRows(rows.context),
  };
  if (typeof value.query === "string" && !widgetRows.exec.length) {
    widgetRows.exec = [{ id: `${id}-row`, query: value.query }];
  }
  const view =
    RESULT_VIEWS.find((v) => v === value.view) ??
    (typeof value.view === "string" ? V1_VIEWS[value.view] : undefined);
  const colors = readColors(value.colors);
  const pins = readPins(value.pins);
  const refresh = value.refreshSeconds;
  return {
    id,
    title: typeof title === "string" ? title : "",
    owner,
    db,
    rows: widgetRows,
    ...(view ? { view } : {}),
    ...(colors ? { colors } : {}),
    ...(pins.length ? { pins } : {}),
    ...(typeof refresh === "number" && isRefreshInterval(refresh)
      ? { refreshSeconds: refresh }
      : {}),
  };
};

const readSize = (value: unknown): WidgetSize => {
  const size = isRecord(value) ? value : {};
  return {
    cols: oneOf(WIDGET_COLS, size.cols, DEFAULT_WIDGET_SIZE.cols),
    rows: oneOf(WIDGET_ROWS, size.rows, DEFAULT_WIDGET_SIZE.rows),
  };
};

const hasWidget = (doc: DashboardDocument, id: string) =>
  Object.hasOwn(doc.widgets, id);

/**
 * Adds widgets at the end of the grid. A widget that is already on the
 * dashboard only gets its content replaced; its place and size stay.
 */
export const addToDocument = (
  doc: DashboardDocument,
  widgets: Widget[],
  size: WidgetSize = DEFAULT_WIDGET_SIZE,
) => {
  for (const widget of widgets) {
    if (!hasWidget(doc, widget.id)) doc.layout.push({ id: widget.id, ...size });
    doc.widgets[widget.id] = widget;
  }
};

export const removeFromDocument = (doc: DashboardDocument, id: string) => {
  delete doc.widgets[id];
  doc.layout = doc.layout.filter((entry) => entry.id !== id);
};

/** Moves a widget by `delta` places (-1 = earlier); moves past either end are ignored. */
export const moveInDocument = (
  doc: DashboardDocument,
  id: string,
  delta: number,
): boolean => {
  const from = doc.layout.findIndex((entry) => entry.id === id);
  const to = from + delta;
  if (from < 0 || to < 0 || to >= doc.layout.length) return false;
  doc.layout.splice(to, 0, ...doc.layout.splice(from, 1));
  return true;
};

export const resizeInDocument = (
  doc: DashboardDocument,
  id: string,
  size: WidgetSize,
): boolean => {
  const entry = doc.layout.find((e) => e.id === id);
  if (!entry) return false;
  entry.cols = size.cols;
  entry.rows = size.rows;
  return true;
};

/** The widgets in grid order, each with its place on the grid. */
export const orderedWidgets = (
  doc: DashboardDocument,
): { widget: Widget; layout: WidgetLayout }[] =>
  doc.layout.flatMap((layout) => {
    const widget = hasWidget(doc, layout.id)
      ? doc.widgets[layout.id]
      : undefined;
    return widget ? [{ widget, layout }] : [];
  });

/**
 * Parses a stored dashboard config. Unknown or broken content gives an empty
 * document; broken widgets are dropped so the rest of the dashboard still loads.
 * Older configs kept a list of widgets, each with its own size. A document
 * from a newer Studio throws `NewerDashboardError`.
 */
export const parseDocument = (config: string): DashboardDocument => {
  let raw: unknown;
  try {
    raw = JSON.parse(config);
  } catch {
    return emptyDocument();
  }
  const doc = emptyDocument();
  if (!isRecord(raw)) return doc;
  // Saving a newer document back as this version would lose its new content.
  if (typeof raw.version === "number" && raw.version > DOCUMENT_VERSION) {
    throw new NewerDashboardError(raw.version);
  }
  if (Array.isArray(raw.widgets)) {
    for (const value of raw.widgets) {
      const widget = readWidget(value);
      if (widget && !hasWidget(doc, widget.id)) {
        addToDocument(doc, [widget], readSize(isRecord(value) && value.size));
      }
    }
    return doc;
  }
  if (!isRecord(raw.widgets)) return doc;
  for (const [id, value] of Object.entries(raw.widgets)) {
    const widget = readWidget(isRecord(value) ? { ...value, id } : value);
    if (widget) doc.widgets[id] = widget;
  }
  for (const entry of Array.isArray(raw.layout) ? raw.layout : []) {
    if (
      isRecord(entry) &&
      typeof entry.id === "string" &&
      hasWidget(doc, entry.id) &&
      !doc.layout.some((e) => e.id === entry.id)
    ) {
      doc.layout.push({ id: entry.id, ...readSize(entry) });
    }
  }
  // Widgets the layout misses still show, at the end.
  for (const id of Object.keys(doc.widgets)) {
    if (!doc.layout.some((e) => e.id === id)) {
      doc.layout.push({ id, ...DEFAULT_WIDGET_SIZE });
    }
  }
  return doc;
};

export const serializeDocument = (doc: DashboardDocument): string =>
  JSON.stringify(doc);

/**
 * Server-side dashboard name rule (`agdb_server/src/routes/dashboard.rs`).
 * No leading '.', so "." and ".." cannot be collapsed out of the URL path.
 */
export const DASHBOARD_NAME_PATTERN = /^[A-Za-z0-9_-][A-Za-z0-9_.-]{0,63}$/;

export const validateDashboardName = (name: string): string | undefined => {
  if (name === "list") return "The name 'list' is reserved";
  if (!DASHBOARD_NAME_PATTERN.test(name)) {
    return "Use 1-64 characters: letters, digits, '_', '-' or '.' (not first)";
  }
  return undefined;
};
