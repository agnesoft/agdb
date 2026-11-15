import type { GraphElement } from "../composables/types";

/**
 * A colour picked from the graph palette, by slot (0-based) so it follows the
 * light/dark theme instead of being a fixed hex value.
 */
export type PaletteSlot = number;

/** Colours elements whose value under `key` matches. */
export type PropertyColorRule = {
  id: string;
  key: string;
  /** "eq": the value equals `value` (as displayed); "exists": the key is present. */
  op: "eq" | "exists";
  value?: string;
  color: PaletteSlot;
};

/** Gives each distinct value under `key` its own palette colour. */
export type PaletteColorRule = {
  id: string;
  key: string;
};

export type ColorRule =
  | ({ kind: "property" } & PropertyColorRule)
  | ({ kind: "palette" } & PaletteColorRule);

/** Colouring of a query's graph; kept with the query. */
export type ColorConfig = {
  /** Applied in order; the first rule that matches wins. */
  rules: ColorRule[];
  /** Colour of the elements returned by a builder row, by row id. */
  rows: Record<string, PaletteSlot>;
  /** Colours set by hand on single elements, by element id. */
  elements: Record<number, PaletteSlot>;
};

export const emptyColorConfig = (): ColorConfig => ({
  rules: [],
  rows: {},
  elements: {},
});

export const hasColoring = (config: ColorConfig | undefined): boolean =>
  !!config &&
  (config.rules.length > 0 ||
    Object.keys(config.rows).length > 0 ||
    Object.keys(config.elements).length > 0);

const valueOf = (element: GraphElement, key: string): string | undefined =>
  element.values.find((v) => v.key === key)?.value;

/** Where a colour comes from, in the order they override each other. */
export type ColorSource = "element" | "property" | "query";

/** One line of the colour legend: a colour, what it means and how many it colours. */
export type LegendEntry = {
  label: string;
  slot: PaletteSlot;
  source: ColorSource;
  count: number;
};

export type ColorResolver = {
  /** The palette slot of an element, or undefined for the default colour. */
  slotOf: (element: GraphElement) => PaletteSlot | undefined;
  /** Only the colours that colour some of the elements, grouped by source. */
  legend: LegendEntry[];
};

export type ColorLabels = {
  row?: (rowId: string) => string;
  element?: (id: number) => string;
};

type Match = { key: string; slot: PaletteSlot };

/**
 * Builds the colouring of the shown elements. Precedence: a colour set by
 * hand, then the first matching rule, then the colour of a row that returned
 * the element. Palette rules give distinct values the palette's colours in
 * order of first appearance; values beyond the palette keep the default
 * colour (colours are never reused for different values).
 * `rowIdsByResult` maps result indexes to the rows that produced them.
 */
export const buildColorResolver = (
  config: ColorConfig | undefined,
  elements: readonly GraphElement[],
  rowIdsByResult: readonly (string | undefined)[],
  paletteSize: number,
  labels: ColorLabels = {},
): ColorResolver => {
  if (!config || !hasColoring(config)) {
    return { slotOf: () => undefined, legend: [] };
  }
  const rowLabel = labels.row ?? ((rowId: string) => rowId);
  const elementLabel = labels.element ?? ((id: number) => `element ${id}`);

  // Legend entries in display order, keyed by what produced them.
  const entries = new Map<string, Omit<LegendEntry, "count">>();
  for (const [id, slot] of Object.entries(config.elements)) {
    entries.set(`e:${id}`, {
      label: elementLabel(Number(id)),
      slot,
      source: "element",
    });
  }
  const paletteSlots = new Map<string, Map<string, PaletteSlot>>();
  for (const rule of config.rules) {
    if (rule.kind === "property") {
      entries.set(`r:${rule.id}`, {
        label:
          rule.op === "exists"
            ? `has ${rule.key}`
            : `${rule.key} = ${rule.value ?? ""}`,
        slot: rule.color,
        source: "property",
      });
      continue;
    }
    const slots = new Map<string, PaletteSlot>();
    for (const element of elements) {
      const value = valueOf(element, rule.key);
      if (value === undefined || slots.has(value)) continue;
      if (slots.size >= paletteSize) break;
      const slot = slots.size;
      slots.set(value, slot);
      entries.set(`r:${rule.id}:${value}`, {
        label: `${rule.key} = ${value}`,
        slot,
        source: "property",
      });
    }
    paletteSlots.set(rule.id, slots);
  }
  for (const [rowId, slot] of Object.entries(config.rows)) {
    entries.set(`q:${rowId}`, {
      label: rowLabel(rowId),
      slot,
      source: "query",
    });
  }

  const ruleMatch = (
    rule: ColorRule,
    element: GraphElement,
  ): Match | undefined => {
    const value = valueOf(element, rule.key);
    if (value === undefined) return undefined;
    if (rule.kind === "palette") {
      const slot = paletteSlots.get(rule.id)?.get(value);
      return slot === undefined
        ? undefined
        : { key: `r:${rule.id}:${value}`, slot };
    }
    if (rule.op === "exists" || value === rule.value) {
      return { key: `r:${rule.id}`, slot: rule.color };
    }
    return undefined;
  };

  const match = (element: GraphElement): Match | undefined => {
    const manual = config.elements[element.id];
    if (manual !== undefined) return { key: `e:${element.id}`, slot: manual };
    for (const rule of config.rules) {
      const found = ruleMatch(rule, element);
      if (found) return found;
    }
    for (const index of element.resultIndexes ?? []) {
      const rowId = rowIdsByResult[index];
      const slot = rowId === undefined ? undefined : config.rows[rowId];
      if (slot !== undefined) return { key: `q:${rowId}`, slot };
    }
    return undefined;
  };

  const counts = new Map<string, number>();
  for (const element of elements) {
    const found = match(element);
    if (found) counts.set(found.key, (counts.get(found.key) ?? 0) + 1);
  }
  const legend = [...entries.entries()].flatMap(([key, entry]) => {
    const count = counts.get(key) ?? 0;
    return count ? [{ ...entry, count }] : [];
  });

  return { slotOf: (element) => match(element)?.slot, legend };
};

/** The colour of a palette slot, if the palette has it. */
export const slotColor = (
  palette: readonly string[],
  slot: PaletteSlot,
): string | undefined => palette[slot];
